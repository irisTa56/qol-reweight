//! The font that draws Japanese text, read from the system.
//!
//! egui's own fonts hold no Japanese glyphs, and the statements of the map's
//! sources are Japanese, so without this font the tool shows no map.

use std::sync::Arc;

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use fontdb::{Database, Family, Query};
use thiserror::Error;

/// The family the font is looked up by. macOS has it.
const FAMILY: &str = "Hiragino Sans";

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

    /// egui's fonts with this one before them, so that it draws every
    /// character it has and they draw the rest. A line of text that two fonts
    /// drew between them would not sit on one baseline.
    pub(crate) fn before_the_defaults(self) -> FontDefinitions {
        let mut fonts = FontDefinitions::default();
        let data = FontData {
            index: self.face,
            ..FontData::from_owned(self.file)
        };
        fonts.font_data.insert(FAMILY.to_owned(), Arc::new(data));
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, FAMILY.to_owned());
        }
        fonts
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

    #[test]
    fn the_font_comes_before_eguis_own() {
        let font = JapaneseFont {
            file: Vec::new(),
            face: 0,
        };
        let fonts = font.before_the_defaults();
        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            assert_eq!(fonts.families[&family][0], FAMILY);
            assert!(fonts.families[&family].len() > 1, "egui's fonts are gone");
        }
    }
}
