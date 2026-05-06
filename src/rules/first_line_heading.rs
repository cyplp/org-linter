use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG008 – The first non-blank, non-directive line should be a heading or #+TITLE.
pub struct FirstLineHeading;

fn is_directive(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("#+") || trimmed.starts_with("#!")
}

impl Rule for FirstLineHeading {
    fn id(&self) -> &str { "ORG008" }
    fn description(&self) -> &str { "First content line should be a heading or #+TITLE directive" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        if doc.lines.is_empty() {
            return vec![];
        }

        // Find first non-blank line
        let first = doc.lines.iter().enumerate().find(|(_, line)| !line.trim().is_empty());
        let Some((line_idx, line)) = first else { return vec![] };

        if heading_level(line).is_some() || is_directive(line) {
            return vec![];
        }

        vec![Violation::new(
            self.id(),
            line_idx + 1,
            "First content line should be a heading or #+TITLE",
        )]
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
    fn heading_first_ok() {
        let document = doc("* Title\n");
        assert!(FirstLineHeading.check(&document).is_empty());
    }

    #[test]
    fn title_directive_ok() {
        let document = doc("#+TITLE: My doc\n* Heading\n");
        assert!(FirstLineHeading.check(&document).is_empty());
    }

    #[test]
    fn plain_text_first_flagged() {
        let document = doc("Some intro text\n* Heading\n");
        assert_eq!(FirstLineHeading.check(&document).len(), 1);
    }

    #[test]
    fn empty_file_ok() {
        let document = doc("");
        assert!(FirstLineHeading.check(&document).is_empty());
    }
}
