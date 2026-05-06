use crate::{document::Document, violation::Violation};
use super::{Rule, heading_level};

/// ORG011 – Every heading of level N must have a direct parent heading of level N-1.
///
/// Uses an ancestor stack to track the current subtree context, so the check
/// is structure-aware: a `***` after `* … ** …` is fine, but a `***` appearing
/// right after a `*` (with no intermediate `**`) is a violation even when they
/// are not consecutive lines.
pub struct SubtreeParent;

impl Rule for SubtreeParent {
    fn id(&self) -> &str { "ORG011" }
    fn description(&self) -> &str {
        "Every heading must have a direct parent exactly one level above"
    }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        // Stack of heading levels representing the current ancestor chain.
        let mut stack: Vec<usize> = Vec::new();

        for (line_idx, line) in doc.lines.iter().enumerate() {
            let Some(level) = heading_level(line) else { continue };

            // Pop all siblings and deeper headings so the top of the stack
            // becomes the direct parent candidate.
            while stack.last().map_or(false, |&stack_top| stack_top >= level) {
                stack.pop();
            }

            if level > 1 {
                match stack.last().copied() {
                    None => {
                        violations.push(Violation::new(
                            self.id(),
                            line_idx + 1,
                            format!(
                                "Level-{level} heading has no parent (expected a level-{} heading above)",
                                level - 1
                            ),
                        ));
                    }
                    Some(parent) if parent != level - 1 => {
                        violations.push(Violation::new(
                            self.id(),
                            line_idx + 1,
                            format!(
                                "Level-{level} heading's direct parent is level {parent} (expected level {})",
                                level - 1
                            ),
                        ));
                    }
                    _ => {}
                }
            }

            stack.push(level);
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
    fn valid_nested_structure() {
        let document = doc("* H1\n** H2\n*** H3\n** H2b\n* H1b\n");
        assert!(SubtreeParent.check(&document).is_empty());
    }

    #[test]
    fn jump_from_one_to_three() {
        let document = doc("* H1\n*** H3\n");
        let violations = SubtreeParent.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(2));
        assert!(violations[0].message.contains("level 2"));
    }

    #[test]
    fn file_starting_at_level_two() {
        // ORG001 misses this — ORG011 catches it
        let document = doc("** H2\n*** H3\n");
        let violations = SubtreeParent.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(1));
    }

    #[test]
    fn jump_after_returning_to_level_one() {
        let document = doc("* H1\n** H2\n* H1b\n*** H3\n");
        let violations = SubtreeParent.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(4));
    }

    #[test]
    fn going_back_up_is_allowed() {
        let document = doc("* A\n** B\n*** C\n* D\n** E\n");
        assert!(SubtreeParent.check(&document).is_empty());
    }

    #[test]
    fn empty_file_no_violation() {
        let document = doc("");
        assert!(SubtreeParent.check(&document).is_empty());
    }

    #[test]
    fn single_top_level_heading_ok() {
        let document = doc("* Title\n");
        assert!(SubtreeParent.check(&document).is_empty());
    }
}
