//! The files under `assets/`: the Japanese text the screen shows, which a
//! source file does not hold.

/// The text of the file named `name` under `assets/`, as the screen shows
/// it: without the line end the file closes with, nor any other ASCII white
/// space at its end, so a text cannot end in a space.
///
/// It is a macro because `include_str!` takes the file's name as written
/// where it is called, which a function's parameter is not. The name is
/// counted from the crate's root, so every module writes it the same way.
macro_rules! text {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name)).trim_ascii_end()
    };
}

pub(crate) use text;

#[cfg(test)]
mod tests {
    /// The file itself ends with a line end, which the text is without.
    #[test]
    fn a_text_is_its_file_without_the_line_end() {
        let file = include_str!("../assets/data-source.txt");
        let text = super::text!("data-source.txt");
        assert_eq!(file.strip_suffix('\n'), Some(text));
        assert!(!text.is_empty());
    }
}
