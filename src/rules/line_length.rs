use crate::{document::Document, violation::Violation};
use super::Rule;

/// ORG005 – Lines should not exceed `max_len` characters.
pub struct LineLength {
    pub max_len: usize,
}

impl Default for LineLength {
    fn default() -> Self { Self { max_len: 120 } }
}

impl Rule for LineLength {
    fn id(&self) -> &str { "ORG005" }
    fn description(&self) -> &str { "Line length should not exceed the maximum" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        doc.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.chars().count() > self.max_len)
            .map(|(line_idx, line)| {
                Violation::new(
                    self.id(),
                    line_idx + 1,
                    format!("Line is {} characters (max {})", line.chars().count(), self.max_len),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::document::Document;

    fn doc(src: &str) -> Document {
        Document::from_content(PathBuf::from("test.org"), src.to_string())
    }

    #[test]
    fn short_line_ok() {
        let document = doc("short\n");
        assert!(LineLength::default().check(&document).is_empty());
    }

    #[test]
    fn long_line_detected() {
        let long = "x".repeat(121);
        let document = doc(&format!("{long}\n"));
        let violations = LineLength::default().check(&document);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn custom_max() {
        let document = doc("hello world\n");
        let rule = LineLength { max_len: 5 };
        assert_eq!(rule.check(&document).len(), 1);
    }
}
