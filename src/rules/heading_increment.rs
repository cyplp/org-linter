use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG001 – Heading levels must increment by at most one at a time.
pub struct HeadingIncrement;

impl Rule for HeadingIncrement {
    fn id(&self) -> &str { "ORG001" }
    fn description(&self) -> &str { "Heading levels should only increment by one level at a time" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut prev_level: Option<usize> = None;

        for (line_idx, line) in doc.lines.iter().enumerate() {
            if let Some(level) = heading_level(line) {
                if let Some(prev) = prev_level {
                    if level > prev + 1 {
                        violations.push(Violation::new(
                            self.id(),
                            line_idx + 1,
                            format!("Heading jumped from level {prev} to level {level}"),
                        ));
                    }
                }
                prev_level = Some(level);
            }
        }
        violations
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
    fn valid_increments() {
        let document = doc("* H1\n** H2\n*** H3\n** back\n* top\n");
        assert!(HeadingIncrement.check(&document).is_empty());
    }

    #[test]
    fn skip_level_detected() {
        let document = doc("* H1\n*** H3 skipped\n");
        let violations = HeadingIncrement.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(2));
    }

    #[test]
    fn going_back_is_ok() {
        let document = doc("* A\n** B\n*** C\n* D\n");
        assert!(HeadingIncrement.check(&document).is_empty());
    }
}
