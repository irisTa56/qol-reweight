//! The font that draws Japanese text, read from the system.
//!
//! egui's own fonts hold no Japanese glyphs, and the statements of the map's
//! sources are Japanese, so without this font the tool shows no map.

use eframe::egui::FontFamily;
use eframe::epaint::text::{FontData, FontInsert, FontPriority, FontTweak, InsertFontFamily};
use fontdb::{Database, Family, Query};
use thiserror::Error;

/// The family the font is looked up by. macOS has it.
const FAMILY: &str = "Hiragino Sans";

/// How far down the font's glyphs are drawn, as a fraction of the text's size.
/// The font's line gap is half its em, and egui makes a row as tall as the
/// ascent, the descent and the gap together, with the whole gap under the
/// descender. Unmoved, the glyphs sit a quarter of an em above the row's
/// middle, in a button as in a label; moved down by half the gap, they sit in
/// the middle.
const Y_OFFSET_FACTOR: f32 = 0.25;

/// A font file's contents, and which of its faces draws Japanese text.
#[derive(Debug)]
pub(crate) struct JapaneseFont {
    file: Vec<u8>,
    face: u32,
}

/// Why there is no font to draw Japanese text with.
#[derive(Debug, Error)]
pub(crate) enum FontError {
    #[error("no font of the family {FAMILY}, which macOS has, is installed")]
    NotInstalled,
    #[error("the file of the font {FAMILY} could not be read")]
    NotRead,
}

impl JapaneseFont {
    /// The font among those installed on the system.
    pub(crate) fn installed() -> Result<Self, FontError> {
        let mut fonts = Database::new();
        fonts.load_system_fonts();
        Self::among(&fonts)
    }

    fn among(fonts: &Database) -> Result<Self, FontError> {
        let query = Query {
            families: &[Family::Name(FAMILY)],
            ..Query::default()
        };
        let id = fonts.query(&query).ok_or(FontError::NotInstalled)?;
        fonts
            .with_face_data(id, |file, face| Self {
                file: file.to_vec(),
                face,
            })
            .ok_or(FontError::NotRead)
    }

    /// The font as egui takes it in, to go before egui's own fonts or after
    /// them. Before them it draws every character it has and they draw the
    /// rest, so that the Latin letters and the digits of a line are in the
    /// typeface of its Japanese text, and not in another beside it.
    pub(crate) fn into_insert(self, priority: FontPriority) -> FontInsert {
        let data = FontData {
            index: self.face,
            tweak: FontTweak {
                y_offset_factor: Y_OFFSET_FACTOR,
                ..FontTweak::default()
            },
            ..FontData::from_owned(self.file)
        };
        let families = [FontFamily::Proportional, FontFamily::Monospace]
            .into_iter()
            .map(|family| InsertFontFamily {
                family,
                priority: priority.clone(),
            })
            .collect();
        FontInsert::new(FAMILY, data, families)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_system_without_the_font_has_none() {
        let fonts = Database::new();
        assert!(matches!(
            JapaneseFont::among(&fonts),
            Err(FontError::NotInstalled)
        ));
    }
}
