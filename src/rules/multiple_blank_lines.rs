use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};

/// ORG004 – No more than `max` consecutive blank lines.
pub struct MultipleBlankLines {
    pub max: usize,
}

impl Default for MultipleBlankLines {
    fn default() -> Self { Self { max: 1 } }
}

impl Rule for MultipleBlankLines {
    fn id(&self) -> &str { "ORG004" }
    fn description(&self) -> &str { "No multiple consecutive blank lines" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        let mut run = 0usize;

        for (line_idx, line) in doc.lines.iter().enumerate() {
            if line.trim().is_empty() {
                run += 1;
                if run > self.max {
                    violations.push(Violation::new(
                        self.id(),
                        line_idx + 1,
                        format!("More than {} consecutive blank line(s)", self.max),
                    ));
                }
            } else {
                run = 0;
            }
        }
        violations
    }
}

impl Fixable for MultipleBlankLines {
    fn fix(&self, doc: &mut Document) -> usize {
        let mut fixed = Vec::with_capacity(doc.lines.len());
        let mut run = 0usize;
        let mut count = 0;

        for line in &doc.lines {
            if line.trim().is_empty() {
                run += 1;
                if run > self.max {
                    count += 1;
                    continue;
                }
            } else {
                run = 0;
            }
            fixed.push(line.clone());
        }

        if count > 0 {
            doc.lines = fixed;
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
    fn single_blank_ok() {
        let document = doc("a\n\nb\n");
        assert!(MultipleBlankLines::default().check(&document).is_empty());
    }

    #[test]
    fn double_blank_detected() {
        let document = doc("a\n\n\nb\n");
        let violations = MultipleBlankLines::default().check(&document);
        assert_eq!(violations.len(), 1);
    }

    #[test]
    fn fix_collapses_blanks() {
        let mut document = doc("a\n\n\n\nb\n");
        let fix_count = MultipleBlankLines::default().fix(&mut document);
        assert_eq!(fix_count, 2);
        assert!(MultipleBlankLines::default().check(&document).is_empty());
    }
}
