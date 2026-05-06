use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub rule: String,
    pub line: Option<usize>,
    pub message: String,
}

impl Violation {
    pub fn new(rule: &str, line: usize, message: impl Into<String>) -> Self {
        Self { rule: rule.to_string(), line: Some(line), message: message.into() }
    }

    pub fn file_level(rule: &str, message: impl Into<String>) -> Self {
        Self { rule: rule.to_string(), line: None, message: message.into() }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(l) => write!(f, "{}:{} [{}] {}", "{path}", l, self.rule, self.message),
            None => write!(f, "{} [{}] {}", "{path}", self.rule, self.message),
        }
    }
}
