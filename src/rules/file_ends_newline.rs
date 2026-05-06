use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};

/// ORG009 – Files should end with exactly one newline.
pub struct FileEndsNewline;

impl Rule for FileEndsNewline {
    fn id(&self) -> &str { "ORG009" }
    fn description(&self) -> &str { "Files should end with a single newline" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        if doc.content.is_empty() {
            return vec![];
        }
        if !doc.content.ends_with('\n') {
            return vec![Violation::file_level(self.id(), "File does not end with a newline")];
        }
        if doc.content.ends_with("\n\n") {
            return vec![Violation::file_level(
                self.id(),
                "File ends with multiple newlines",
            )];
        }
        vec![]
    }
}

impl Fixable for FileEndsNewline {
    fn fix(&self, doc: &mut Document) -> usize {
        if doc.content.is_empty() {
            return 0;
        }
        let trimmed = doc.content.trim_end_matches('\n').to_string();
        let fixed = format!("{trimmed}\n");
        if fixed != doc.content {
            // Rebuild lines from corrected content
            doc.lines = trimmed.lines().map(|line| line.to_string()).collect();
            doc.content = fixed;
            return 1;
        }
        0
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
    fn proper_newline_ok() {
        let document = doc("* Heading\n");
        assert!(FileEndsNewline.check(&document).is_empty());
    }

    #[test]
    fn missing_newline_flagged() {
        let document = doc("* Heading");
        assert_eq!(FileEndsNewline.check(&document).len(), 1);
    }

    #[test]
    fn double_newline_flagged() {
        let document = doc("* Heading\n\n");
        assert_eq!(FileEndsNewline.check(&document).len(), 1);
    }

    #[test]
    fn fix_adds_newline() {
        let mut document = doc("* Heading");
        FileEndsNewline.fix(&mut document);
        assert!(FileEndsNewline.check(&document).is_empty());
    }

    #[test]
    fn fix_removes_extra_newlines() {
        let mut document = doc("* Heading\n\n\n");
        FileEndsNewline.fix(&mut document);
        assert!(FileEndsNewline.check(&document).is_empty());
    }
}
