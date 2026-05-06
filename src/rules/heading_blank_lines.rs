use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG006 – Headings must be preceded and followed by a blank line
/// (except at the very start/end of the file).
pub struct HeadingBlankLines;

impl Rule for HeadingBlankLines {
    fn id(&self) -> &str { "ORG006" }
    fn description(&self) -> &str { "Headings should be surrounded by blank lines" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        let lines = &doc.lines;

        for (line_idx, line) in lines.iter().enumerate() {
            if heading_level(line).is_none() {
                continue;
            }

            // Check blank line before (skip first line)
            if line_idx > 0 {
                let prev = &lines[line_idx - 1];
                if !prev.trim().is_empty() && heading_level(prev).is_none() {
                    violations.push(Violation::new(
                        self.id(),
                        line_idx + 1,
                        "Heading should be preceded by a blank line",
                    ));
                }
            }

            // Check blank line after (skip last line)
            if line_idx + 1 < lines.len() {
                let next = &lines[line_idx + 1];
                if !next.trim().is_empty() && heading_level(next).is_none() {
                    violations.push(Violation::new(
                        self.id(),
                        line_idx + 1,
                        "Heading should be followed by a blank line",
                    ));
                }
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
    fn well_surrounded_heading() {
        let document = doc("* Title\n\nsome text\n\n** Sub\n\nmore\n");
        assert!(HeadingBlankLines.check(&document).is_empty());
    }

    #[test]
    fn heading_at_start_no_blank_needed_before() {
        let document = doc("* Title\n\ntext\n");
        assert!(HeadingBlankLines.check(&document).is_empty());
    }

    #[test]
    fn missing_blank_after_heading() {
        let document = doc("* Title\ntext here\n");
        let violations = HeadingBlankLines.check(&document);
        assert!(!violations.is_empty());
    }

    #[test]
    fn missing_blank_before_heading() {
        let document = doc("text\n** Sub\n");
        let violations = HeadingBlankLines.check(&document);
        assert!(!violations.is_empty());
    }

    #[test]
    fn consecutive_headings_ok() {
        let document = doc("* A\n** B\n*** C\n");
        assert!(HeadingBlankLines.check(&document).is_empty());
    }
}
