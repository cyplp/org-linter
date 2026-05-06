use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};

/// ORG002 – No trailing whitespace.
pub struct TrailingSpaces;

impl Rule for TrailingSpaces {
    fn id(&self) -> &str { "ORG002" }
    fn description(&self) -> &str { "No trailing spaces" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        doc.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.ends_with(|ch: char| ch == ' ' || ch == '\t'))
            .map(|(line_idx, _)| Violation::new(self.id(), line_idx + 1, "Trailing whitespace"))
            .collect()
    }
}

impl Fixable for TrailingSpaces {
    fn fix(&self, doc: &mut Document) -> usize {
        let mut count = 0;
        for line in &mut doc.lines {
            let trimmed = line.trim_end().to_string();
            if trimmed != *line {
                *line = trimmed;
                count += 1;
            }
        }
        if count > 0 {
            doc.content = doc.to_content();
        }
        count
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
    fn detects_trailing_space() {
        let document = doc("line one   \nline two\n");
        let violations = TrailingSpaces.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(1));
    }

    #[test]
    fn no_false_positive() {
        let document = doc("* Heading\n\ntext\n");
        assert!(TrailingSpaces.check(&document).is_empty());
    }

    #[test]
    fn fix_removes_trailing_spaces() {
        let mut document = doc("hello   \nworld  \n");
        let count = TrailingSpaces.fix(&mut document);
        assert_eq!(count, 2);
        assert!(TrailingSpaces.check(&document).is_empty());
    }
}
