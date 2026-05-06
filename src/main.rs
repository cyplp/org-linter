use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

use org_linter::{
    linter::{add_noqa_to_file, check_file, fix_file},
    walker::collect_org_files,
};

#[derive(Parser)]
#[command(
    name = "org-linter",
    about = "A markdownlint-style linter for org-mode files",
    version
)]
struct Cli {
    /// Files or directories to lint
    #[arg(required_unless_present = "list_rules")]
    paths: Vec<PathBuf>,

    /// Check files and exit with error if violations are found
    #[arg(long, conflicts_with_all = ["fix", "add_noqa", "stats"])]
    check: bool,

    /// Fix trivially auto-fixable violations in place
    #[arg(long, conflicts_with_all = ["check", "add_noqa", "stats"])]
    fix: bool,

    /// Add # noqa comments on violating lines for the selected rules
    #[arg(long, conflicts_with_all = ["check", "fix", "stats"])]
    add_noqa: bool,

    /// Print an org-mode table of violation counts per rule
    #[arg(long, conflicts_with_all = ["check", "fix", "add_noqa"])]
    stats: bool,

    /// Only process these rule IDs, comma-separated (default: all rules)
    #[arg(long, value_delimiter = ',')]
    select: Vec<String>,

    /// Show rule descriptions
    #[arg(long)]
    list_rules: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.list_rules {
        print_rules();
        return;
    }

    let files: Vec<PathBuf> = cli
        .paths
        .iter()
        .flat_map(|dir_path| collect_org_files(dir_path))
        .collect();

    if files.is_empty() {
        eprintln!("No .org files found.");
        process::exit(0);
    }

    let selected = &cli.select;

    if cli.add_noqa {
        run_add_noqa(&files, selected);
    } else if cli.fix {
        run_fix(&files, selected);
    } else if cli.stats {
        run_stats(&files, selected);
    } else {
        let has_violations = run_check(&files, selected);
        if cli.check && has_violations {
            process::exit(1);
        }
    }
}

fn run_check(files: &[PathBuf], selected: &[String]) -> bool {
    let mut has_violations = false;
    for path in files {
        match check_file(path, selected) {
            Ok(result) => {
                result.print();
                if !result.is_clean() {
                    has_violations = true;
                }
            }
            Err(err) => {
                eprintln!("{}: error reading file: {err}", path.display());
                has_violations = true;
            }
        }
    }
    has_violations
}

fn run_fix(files: &[PathBuf], selected: &[String]) {
    for path in files {
        match fix_file(path, selected) {
            Ok(0) => {}
            Ok(fix_count) => println!("{}: fixed {fix_count} issue(s)", path.display()),
            Err(err) => eprintln!("{}: error: {err}", path.display()),
        }
    }
}

fn run_add_noqa(files: &[PathBuf], selected: &[String]) {
    for path in files {
        match add_noqa_to_file(path, selected) {
            Ok(0) => {}
            Ok(noqa_count) => println!("{}: added {noqa_count} noqa comment(s)", path.display()),
            Err(err) => eprintln!("{}: error: {err}", path.display()),
        }
    }
}

fn run_stats(files: &[PathBuf], selected: &[String]) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for path in files {
        match check_file(path, selected) {
            Ok(result) => {
                for violation in result.violations {
                    *counts.entry(violation.rule).or_insert(0) += 1;
                }
            }
            Err(err) => eprintln!("{}: error reading file: {err}", path.display()),
        }
    }
    print_org_stats_table(&counts);
}

fn print_org_stats_table(counts: &HashMap<String, usize>) {
    let mut rows: Vec<(&str, usize)> = counts
        .iter()
        .map(|(rule, &count)| (rule.as_str(), count))
        .collect();
    rows.sort_by(|(rule_a, count_a), (rule_b, count_b)| count_b.cmp(count_a).then(rule_a.cmp(rule_b)));

    let rule_col_width = rows.iter()
        .map(|(rule, _)| rule.len())
        .max()
        .unwrap_or(0)
        .max("Rule".len());

    let count_col_width = rows.iter()
        .map(|(_, count)| count.to_string().len())
        .max()
        .unwrap_or(0)
        .max("Occurrences".len());

    let header_rule  = format!("{:<width$}", "Rule",        width = rule_col_width);
    let header_count = format!("{:>width$}", "Occurrences", width = count_col_width);
    println!("| {} | {} |", header_rule, header_count);
    println!("|{}+{}|", "-".repeat(rule_col_width + 2), "-".repeat(count_col_width + 2));

    for (rule, count) in &rows {
        let cell_rule  = format!("{:<width$}", rule,  width = rule_col_width);
        let cell_count = format!("{:>width$}", count, width = count_col_width);
        println!("| {} | {} |", cell_rule, cell_count);
    }
}

fn print_rules() {
    use org_linter::rules::all_rules;
    println!("{:<8} {}", "ID", "Description");
    println!("{}", "-".repeat(60));
    for rule in all_rules() {
        println!("{:<8} {}", rule.id(), rule.description());
    }
}
