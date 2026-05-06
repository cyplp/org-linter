use crate::{document::Document, violation::Violation};

/// Every lint rule implements this trait (SRP + OCP).
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, doc: &Document) -> Vec<Violation>;
}

/// Rules that can automatically repair violations.
pub trait Fixable: Rule {
    fn fix(&self, doc: &mut Document) -> usize;
}

mod heading_increment;
mod trailing_spaces;
mod hard_tabs;
mod multiple_blank_lines;
mod line_length;
mod heading_blank_lines;
mod single_top_heading;
mod first_line_heading;
mod file_ends_newline;
mod has_heading;
mod subtree_parent;
mod table_format;
mod bare_url;

pub use heading_increment::HeadingIncrement;
pub use trailing_spaces::TrailingSpaces;
pub use hard_tabs::HardTabs;
pub use multiple_blank_lines::MultipleBlankLines;
pub use line_length::LineLength;
pub use heading_blank_lines::HeadingBlankLines;
pub use single_top_heading::SingleTopHeading;
pub use first_line_heading::FirstLineHeading;
pub use file_ends_newline::FileEndsNewline;
pub use has_heading::HasHeading;
pub use subtree_parent::SubtreeParent;
pub use table_format::TableFormat;
pub use bare_url::BareUrl;

/// Build the default ordered list of all rules.
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(HeadingIncrement),
        Box::new(TrailingSpaces),
        Box::new(HardTabs),
        Box::new(MultipleBlankLines::default()),
        Box::new(LineLength::default()),
        Box::new(HeadingBlankLines),
        Box::new(SingleTopHeading),
        Box::new(FirstLineHeading),
        Box::new(FileEndsNewline),
        Box::new(HasHeading),
        Box::new(SubtreeParent),
        Box::new(TableFormat),
        Box::new(BareUrl::default()),
    ]
}

/// Build the list of fixable rules (returns trait objects via a helper enum).
pub fn fixable_rules() -> Vec<Box<dyn FixableRule>> {
    vec![
        Box::new(TrailingSpaces),
        Box::new(HardTabs),
        Box::new(MultipleBlankLines::default()),
        Box::new(FileEndsNewline),
        Box::new(TableFormat),
        Box::new(BareUrl::default()),
    ]
}

/// Combines both traits so we can store fixable rules as a single trait object.
pub trait FixableRule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, doc: &Document) -> Vec<Violation>;
    fn fix(&self, doc: &mut Document) -> usize;
}

macro_rules! impl_fixable_rule {
    ($t:ty) => {
        impl FixableRule for $t {
            fn id(&self) -> &str { Rule::id(self) }
            fn check(&self, doc: &Document) -> Vec<Violation> { Rule::check(self, doc) }
            fn fix(&self, doc: &mut Document) -> usize { Fixable::fix(self, doc) }
        }
    };
}

impl_fixable_rule!(TrailingSpaces);
impl_fixable_rule!(HardTabs);
impl_fixable_rule!(MultipleBlankLines);
impl_fixable_rule!(FileEndsNewline);
impl_fixable_rule!(TableFormat);
impl_fixable_rule!(BareUrl);

pub fn heading_level(line: &str) -> Option<usize> {
    if line.starts_with('*') {
        let level = line.chars().take_while(|&ch| ch == '*').count();
        let rest = &line[level..];
        if rest.is_empty() || rest.starts_with(' ') {
            return Some(level);
        }
    }
    None
}
