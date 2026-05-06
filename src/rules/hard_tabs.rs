use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};

/// ORG003 – No hard tab characters.
pub struct HardTabs;

impl Rule for HardTabs {
    fn id(&self) -> &str { "ORG003" }
    fn description(&self) -> &str { "No hard tab characters" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        doc.lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains('\t'))
            .map(|(line_idx, _)| Violation::new(self.id(), line_idx + 1, "Hard tab character found"))
            .collect()
    }
}

impl Fixable for HardTabs {
    fn fix(&self, doc: &mut Document) -> usize {
        let mut count = 0;
        for line in &mut doc.lines {
            if line.contains('\t') {
                *line = line.replace('\t', "    ");
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
    fn detects_tab() {
        let document = doc("* H\n\tcontent\n");
        let violations = HardTabs.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(2));
    }

    #[test]
    fn fix_replaces_tabs() {
        let mut document = doc("\tindented\n");
        HardTabs.fix(&mut document);
        assert!(!document.lines[0].contains('\t'));
        assert!(document.lines[0].starts_with("    "));
    }
}
