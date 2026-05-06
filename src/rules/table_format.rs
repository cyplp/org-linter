use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};
use std::ops::Range;

fn is_table_row(line: &str) -> bool {
    line.trim_start().starts_with('|')
}

fn is_separator_row(line: &str) -> bool {
    if !is_table_row(line) { return false; }
    line.trim_start().chars().all(|ch| matches!(ch, '|' | '-' | '+' | ' '))
}

fn parse_cells(line: &str) -> Vec<String> {
    let trimmed_row = line.trim_start();
    let without_prefix = trimmed_row.strip_prefix('|').unwrap_or(trimmed_row);
    let inner = without_prefix.strip_suffix('|').unwrap_or(without_prefix);
    inner.split('|').map(|cell_text| cell_text.to_string()).collect()
}

fn collect_table_blocks(lines: &[String]) -> Vec<Range<usize>> {
    let mut blocks = Vec::new();
    let mut idx = 0;
    while idx < lines.len() {
        if is_table_row(&lines[idx]) {
            let start = idx;
            while idx < lines.len() && is_table_row(&lines[idx]) { idx += 1; }
            blocks.push(start..idx);
        } else {
            idx += 1;
        }
    }
    blocks
}

fn compute_col_widths(lines: &[String], block: &Range<usize>) -> Vec<usize> {
    let mut widths: Vec<usize> = Vec::new();
    for line in &lines[block.start..block.end] {
        if is_separator_row(line) { continue; }
        for (col, cell) in parse_cells(line).iter().enumerate() {
            let content_width = cell.trim().len();
            if col >= widths.len() { widths.resize(col + 1, 0); }
            if content_width > widths[col] { widths[col] = content_width; }
        }
    }
    widths
}

fn format_data_row(cells: &[String], widths: &[usize]) -> String {
    let mut out = String::from("|");
    for (col, &col_width) in widths.iter().enumerate() {
        let content = cells.get(col).map(|raw_cell| raw_cell.trim().to_string()).unwrap_or_default();
        out.push_str(&format!(" {:<width$} |", content, width = col_width));
    }
    out
}

fn format_separator_row(widths: &[usize]) -> String {
    let mut out = String::from("|");
    for (col_idx, &col_width) in widths.iter().enumerate() {
        out.push_str(&"-".repeat(col_width + 2));
        if col_idx + 1 < widths.len() { out.push('+'); }
    }
    out.push('|');
    out
}

pub struct TableFormat;

impl Rule for TableFormat {
    fn id(&self) -> &str { "ORG012" }
    fn description(&self) -> &str { "Table columns must have uniform width with at least one space of padding" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        let mut violations = Vec::new();
        for block in collect_table_blocks(&doc.lines) {
            let widths = compute_col_widths(&doc.lines, &block);
            if widths.is_empty() { continue; }
            for line_idx in block {
                let line = &doc.lines[line_idx];
                let line_no = line_idx + 1;
                if is_separator_row(line) {
                    if line.trim_start() != format_separator_row(&widths) {
                        violations.push(Violation::new(self.id(), line_no,
                            "Separator row is not in canonical form"));
                    }
                } else {
                    let cells = parse_cells(line);
                    for col in 0..widths.len() {
                        let cell = cells.get(col).map(|raw_cell| raw_cell.as_str()).unwrap_or("");
                        if !cell.trim().is_empty() && (!cell.starts_with(' ') || !cell.ends_with(' ')) {
                            violations.push(Violation::new(self.id(), line_no,
                                format!("Column {} cell missing surrounding spaces", col + 1)));
                            break;
                        }
                        if cell.len() != widths[col] + 2 {
                            violations.push(Violation::new(self.id(), line_no,
                                format!("Column {} cell width {} != expected {}",
                                    col + 1, cell.len(), widths[col] + 2)));
                            break;
                        }
                    }
                }
            }
        }
        violations
    }
}

impl Fixable for TableFormat {
    fn fix(&self, doc: &mut Document) -> usize {
        let blocks = collect_table_blocks(&doc.lines);
        let mut count = 0;
        for block in blocks {
            let widths = compute_col_widths(&doc.lines, &block);
            if widths.is_empty() { continue; }
            for line_idx in block {
                let new_line = if is_separator_row(&doc.lines[line_idx]) {
                    format_separator_row(&widths)
                } else {
                    let cells = parse_cells(&doc.lines[line_idx]);
                    format_data_row(&cells, &widths)
                };
                if new_line != doc.lines[line_idx] {
                    doc.lines[line_idx] = new_line;
                    count += 1;
                }
            }
        }
        if count > 0 { doc.content = doc.to_content(); }
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
    fn canonical_no_violations() {
        let document = doc("| foo | bar |\n|-----+-----|\n| baz | qux |\n");
        assert!(TableFormat.check(&document).is_empty());
        let mut document = document;
        assert_eq!(TableFormat.fix(&mut document), 0);
    }

    #[test]
    fn missing_left_space() {
        let document = doc("|foo | bar |\n");
        let violations = TableFormat.check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(1));
        assert!(violations[0].message.contains("Column 1") && violations[0].message.contains("missing surrounding spaces"));
    }

    #[test]
    fn missing_right_space() {
        let document = doc("| foo| bar |\n");
        let violations = TableFormat.check(&document);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("Column 1") && violations[0].message.contains("missing surrounding spaces"));
    }

    #[test]
    fn unequal_widths() {
        let document = doc("| foo |\n| a longer value |\n");
        let violations = TableFormat.check(&document);
        assert!(!violations.is_empty());
        assert_eq!(violations[0].line, Some(1));
        assert!(violations[0].message.contains("Column 1"));
    }

    #[test]
    fn fix_normalizes_padding() {
        let mut document = doc("|foo|bar|\n");
        let count = TableFormat.fix(&mut document);
        assert!(count > 0);
        assert_eq!(document.lines[0], "| foo | bar |");
        assert!(TableFormat.check(&document).is_empty());
    }

    #[test]
    fn fix_normalizes_widths() {
        let mut document = doc("| Name | Value |\n| x | 123456 |\n");
        TableFormat.fix(&mut document);
        assert_eq!(document.lines[0], "| Name | Value  |");
        assert_eq!(document.lines[1], "| x    | 123456 |");
        assert!(TableFormat.check(&document).is_empty());
    }

    #[test]
    fn fix_normalizes_separator() {
        let mut document = doc("| foo | bar |\n|--+--|\n| baz | qux |\n");
        TableFormat.fix(&mut document);
        assert_eq!(document.lines[1], "|-----+-----|");
        assert!(TableFormat.check(&document).is_empty());
    }

    #[test]
    fn multi_table_file() {
        let mut document = doc("| a | b |\n| c | d |\n\n| x | y |\n| z | w |\n");
        let count = TableFormat.fix(&mut document);
        assert_eq!(count, 0); // already canonical
        let document2 = doc("|a|b|\n|c|d|\n\nnot a table\n|x|y|\n");
        let violations = TableFormat.check(&document2);
        assert!(!violations.is_empty());
    }

    #[test]
    fn empty_cell_handling() {
        let mut document = doc("| foo |  |\n| bar | baz |\n");
        TableFormat.fix(&mut document);
        assert_eq!(document.lines[0], "| foo |     |");
        assert!(TableFormat.check(&document).is_empty());
    }

    #[test]
    fn single_column_no_plus() {
        let mut document = doc("| only column |\n|---|\n| data |\n");
        TableFormat.fix(&mut document);
        assert_eq!(document.lines[1], "|-------------|");
        assert!(!document.lines[1].contains('+'));
        assert!(TableFormat.check(&document).is_empty());
    }

    #[test]
    fn fix_returns_line_count() {
        let mut document = doc("|a|\n|b|\n|c|\n");
        let count = TableFormat.fix(&mut document);
        assert_eq!(count, 3);
    }

    #[test]
    fn non_table_ignored() {
        let document = doc("* Heading\n\nSome text here.\n\n** Subsection\n");
        assert!(TableFormat.check(&document).is_empty());
        let mut document = document;
        assert_eq!(TableFormat.fix(&mut document), 0);
    }
}
