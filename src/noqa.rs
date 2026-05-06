use std::collections::{HashMap, HashSet};

pub struct NoqaMap {
    inline: HashMap<usize, HashSet<String>>,
    file_level: Vec<(usize, HashSet<String>)>,
}

impl NoqaMap {
    pub fn from_lines(lines: &[String]) -> Self {
        let mut inline = HashMap::new();
        let mut file_level = Vec::new();
        for (idx, line) in lines.iter().enumerate() {
            let line_no = idx + 1;
            let trimmed = line.trim_start();
            if trimmed.starts_with("# noqa:") {
                file_level.push((line_no, parse_rule_list(&trimmed["# noqa:".len()..])));
            } else if let Some(pos) = line.find("# noqa:") {
                inline.insert(line_no, parse_rule_list(&line[pos + "# noqa:".len()..]));
            }
        }
        Self { inline, file_level }
    }

    pub fn is_disabled(&self, rule_id: &str, line: Option<usize>) -> bool {
        match line {
            Some(line_no) => {
                self.inline.get(&line_no).is_some_and(|rule_set| rule_set.contains(rule_id))
                    || self.file_level.iter().any(|(from, rule_set)| line_no >= *from && rule_set.contains(rule_id))
            }
            None => self.file_level.iter().any(|(_, rule_set)| rule_set.contains(rule_id)),
        }
    }
}

fn parse_rule_list(raw: &str) -> HashSet<String> {
    raw.split(',').map(|entry| entry.trim().to_string()).filter(|rule_id| !rule_id.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(src: &str) -> Vec<String> {
        src.lines().map(String::from).collect()
    }

    #[test]
    fn no_noqa_no_suppression() {
        let noqa_map = NoqaMap::from_lines(&lines("some text\nother text\n"));
        assert!(!noqa_map.is_disabled("ORG001", Some(1)));
        assert!(!noqa_map.is_disabled("ORG001", None));
    }

    #[test]
    fn inline_disables_rule_for_line() {
        let noqa_map = NoqaMap::from_lines(&lines("text  # noqa: ORG001\nother text\n"));
        assert!(noqa_map.is_disabled("ORG001", Some(1)));
        assert!(!noqa_map.is_disabled("ORG001", Some(2)));
    }

    #[test]
    fn inline_does_not_affect_other_rules() {
        let noqa_map = NoqaMap::from_lines(&lines("text  # noqa: ORG001\n"));
        assert!(!noqa_map.is_disabled("ORG002", Some(1)));
    }

    #[test]
    fn multiple_rules_in_one_directive() {
        let noqa_map = NoqaMap::from_lines(&lines("text  # noqa: ORG001, ORG002\n"));
        assert!(noqa_map.is_disabled("ORG001", Some(1)));
        assert!(noqa_map.is_disabled("ORG002", Some(1)));
        assert!(!noqa_map.is_disabled("ORG003", Some(1)));
    }

    #[test]
    fn file_level_disables_from_line() {
        let noqa_map = NoqaMap::from_lines(&lines("line1\nline2\n# noqa: ORG001\nline4\n"));
        assert!(!noqa_map.is_disabled("ORG001", Some(1)));
        assert!(!noqa_map.is_disabled("ORG001", Some(2)));
        assert!(noqa_map.is_disabled("ORG001", Some(3)));
        assert!(noqa_map.is_disabled("ORG001", Some(4)));
    }

    #[test]
    fn file_level_does_not_affect_other_rules() {
        let noqa_map = NoqaMap::from_lines(&lines("# noqa: ORG001\n"));
        assert!(!noqa_map.is_disabled("ORG002", Some(1)));
    }

    #[test]
    fn file_level_violation_suppressed() {
        let noqa_map = NoqaMap::from_lines(&lines("# noqa: ORG010\n"));
        assert!(noqa_map.is_disabled("ORG010", None));
    }

    #[test]
    fn file_level_violation_not_suppressed_by_inline() {
        let noqa_map = NoqaMap::from_lines(&lines("text  # noqa: ORG010\n"));
        assert!(!noqa_map.is_disabled("ORG010", None));
    }
}
