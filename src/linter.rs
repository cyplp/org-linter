use std::collections::HashMap;
use std::path::Path;

use crate::{
    document::Document,
    noqa::NoqaMap,
    rules::{all_rules, fixable_rules, Rule, FixableRule},
    violation::Violation,
};

pub struct LintResult {
    pub path: String,
    pub violations: Vec<Violation>,
}

impl LintResult {
    pub fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }

    pub fn print(&self) {
        for violation in &self.violations {
            match violation.line {
                Some(line_no) => eprintln!("{}:{} [{}] {}", self.path, line_no, violation.rule, violation.message),
                None => eprintln!("{} [{}] {}", self.path, violation.rule, violation.message),
            }
        }
    }
}

/// Run selected rules on a single file and return violations (noqa-filtered).
/// Pass an empty slice to run all rules.
pub fn check_file(path: &Path, selected: &[String]) -> std::io::Result<LintResult> {
    let doc = Document::from_path(path)?;
    let noqa = NoqaMap::from_lines(&doc.lines);
    let violations = filter_rules(selected)
        .iter()
        .flat_map(|rule| {
            rule.check(&doc)
                .into_iter()
                .filter(|violation| !noqa.is_disabled(rule.id(), violation.line))
        })
        .collect();
    Ok(LintResult { path: path.display().to_string(), violations })
}

/// Apply selected fixable rules to a file and write it back if changed.
/// Returns the number of fixes applied.
pub fn fix_file(path: &Path, selected: &[String]) -> std::io::Result<usize> {
    let mut doc = Document::from_path(path)?;
    let noqa = NoqaMap::from_lines(&doc.lines);
    let mut total = 0;

    for rule in filter_fixable_rules(selected) {
        let before = doc.lines.clone();
        let fixed = rule.fix(&mut doc);
        if fixed > 0 {
            let mut reverted = 0;
            for (line_idx, (orig_line, current_line)) in before.iter().zip(doc.lines.iter_mut()).enumerate() {
                if orig_line != current_line && noqa.is_disabled(rule.id(), Some(line_idx + 1)) {
                    *current_line = orig_line.clone();
                    reverted += 1;
                }
            }
            if reverted > 0 {
                doc.content = doc.to_content();
            }
            total += fixed - reverted;
        }
    }

    if total > 0 {
        std::fs::write(path, &doc.content)?;
    }
    Ok(total)
}

/// Add `# noqa: RULE` comments to every line that violates a selected rule.
/// File-level violations produce a directive at the top of the file.
/// Returns the number of lines modified.
pub fn add_noqa_to_file(path: &Path, selected: &[String]) -> std::io::Result<usize> {
    let mut doc = Document::from_path(path)?;
    let violations: Vec<Violation> = filter_rules(selected)
        .iter()
        .flat_map(|rule| rule.check(&doc).into_iter())
        .collect();
    if violations.is_empty() {
        return Ok(0);
    }
    let count = apply_noqa_comments(&mut doc, &violations);
    if count > 0 {
        std::fs::write(path, &doc.content)?;
    }
    Ok(count)
}

fn filter_rules(selected: &[String]) -> Vec<Box<dyn Rule>> {
    let rules = all_rules();
    if selected.is_empty() {
        rules
    } else {
        rules.into_iter()
            .filter(|rule| selected.iter().any(|id| id == rule.id()))
            .collect()
    }
}

fn filter_fixable_rules(selected: &[String]) -> Vec<Box<dyn FixableRule>> {
    let rules = fixable_rules();
    if selected.is_empty() {
        rules
    } else {
        rules.into_iter()
            .filter(|rule| selected.iter().any(|id| id == rule.id()))
            .collect()
    }
}

fn apply_noqa_comments(doc: &mut Document, violations: &[Violation]) -> usize {
    // Aggregate rule IDs per line (None = file-level violation)
    let mut by_line: HashMap<Option<usize>, Vec<String>> = HashMap::new();
    for violation in violations {
        let entry = by_line.entry(violation.line).or_default();
        if !entry.contains(&violation.rule) {
            entry.push(violation.rule.clone());
        }
    }
    for rule_ids in by_line.values_mut() {
        rule_ids.sort();
    }

    let mut count = 0;

    // Inline violations: modify the existing line (no index shift)
    for (line_opt, rule_ids) in &by_line {
        if let Some(line_no) = *line_opt {
            let idx = line_no - 1;
            if idx < doc.lines.len() {
                let modified = merge_noqa_comment(&doc.lines[idx], rule_ids);
                if modified != doc.lines[idx] {
                    doc.lines[idx] = modified;
                    count += 1;
                }
            }
        }
    }

    // File-level violations: insert or merge at the top of the file
    if let Some(rule_ids) = by_line.get(&None) {
        if doc.lines.first().map(|line| line.trim_start().starts_with("# noqa:")).unwrap_or(false) {
            let merged = merge_noqa_comment(&doc.lines[0], rule_ids);
            if merged != doc.lines[0] {
                doc.lines[0] = merged;
                count += 1;
            }
        } else {
            doc.lines.insert(0, format!("# noqa: {}", rule_ids.join(", ")));
            count += 1;
        }
    }

    if count > 0 {
        doc.content = doc.to_content();
    }
    count
}

/// Append or merge rule IDs into the `# noqa:` comment of a line.
/// If no `# noqa:` exists, two spaces and the comment are appended.
fn merge_noqa_comment(line: &str, rule_ids: &[String]) -> String {
    if let Some(pos) = line.find("# noqa:") {
        let prefix = &line[..pos + "# noqa:".len()];
        let existing_raw = &line[pos + "# noqa:".len()..];
        let mut existing: Vec<String> = existing_raw
            .split(',')
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty())
            .collect();
        for rule_id in rule_ids {
            if !existing.contains(rule_id) {
                existing.push(rule_id.clone());
            }
        }
        existing.sort();
        format!("{} {}", prefix, existing.join(", "))
    } else {
        format!("{}  # noqa: {}", line, rule_ids.join(", "))
    }
}
