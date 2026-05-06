use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG007 – There should be only one top-level heading per file.
pub struct SingleTopHeading;

impl Rule for SingleTopHeading {
    fn id(&self) -> &str { "ORG007" }
    fn description(&self) -> &str { "Only one top-level heading allowed per file" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let top_headings: Vec<usize> = doc.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| heading_level(line) == Some(1))
            .map(|(line_idx, _)| line_idx + 1)
            .collect();

        if top_headings.len() <= 1 {
            return vec![];
        }

        top_headings[1..]
            .iter()
            .map(|&line_no| Violation::new(
                self.id(),
                line_no,
                "Multiple top-level headings in the same document",
            ))
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
    fn single_top_ok() {
        let document = doc("* Title\n\n** Sub\n");
        assert!(SingleTopHeading.check(&document).is_empty());
    }

    #[test]
    fn two_top_headings_flagged() {
        let document = doc("* First\n\n* Second\n");
        let violations = SingleTopHeading.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(3));
    }
}
