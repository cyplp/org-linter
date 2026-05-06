use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG010 – Every org file must contain at least one heading.
pub struct HasHeading;

impl Rule for HasHeading {
    fn id(&self) -> &str { "ORG010" }
    fn description(&self) -> &str { "File must contain at least one heading" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        if doc.lines.is_empty() {
            return vec![];
        }
        if doc.lines.iter().any(|line| heading_level(line).is_some()) {
            return vec![];
        }
        vec![Violation::file_level(self.id(), "File contains no headings")]
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
    fn empty_file_no_violation() {
        let document = doc("");
        assert!(HasHeading.check(&document).is_empty());
    }

    #[test]
    fn only_title_directive_flagged() {
        let document = doc("#+TITLE: My doc\n\nsome text\n");
        let violations = HasHeading.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "ORG010");
    }

    #[test]
    fn top_level_heading_ok() {
        let document = doc("* Mon titre\n\ntexte\n");
        assert!(HasHeading.check(&document).is_empty());
    }

    #[test]
    fn sub_heading_only_ok() {
        let document = doc("** Sous-titre\n\ntexte\n");
        assert!(HasHeading.check(&document).is_empty());
    }

    #[test]
    fn malformed_heading_not_counted() {
        // "*sans espace" is not a valid org heading
        let document = doc("*sansespace\n\ntexte\n");
        let violations = HasHeading.check(&document);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn plain_text_only_flagged() {
        let document = doc("just some text\nno heading here\n");
        assert_eq!(HasHeading.check(&document).len(), 1);
    }
}
