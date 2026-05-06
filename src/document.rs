use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub content: String,
    pub lines: Vec<String>,
}

impl Document {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_content(path.to_path_buf(), content))
    }

    pub fn from_content(path: PathBuf, content: String) -> Self {
        let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
        Self { path, content, lines }
    }

    pub fn to_content(&self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }
        let joined = self.lines.join("\n");
        if self.content.ends_with('\n') {
            format!("{joined}\n")
        } else {
            joined
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_trailing_newline() {
        let content = "* Heading\n\nsome text\n".to_string();
        let doc = Document::from_content(PathBuf::from("test.org"), content.clone());
        assert_eq!(doc.to_content(), content);
    }

    #[test]
    fn roundtrip_no_trailing_newline() {
        let content = "* Heading\n\nsome text".to_string();
        let doc = Document::from_content(PathBuf::from("test.org"), content.clone());
        assert_eq!(doc.to_content(), content);
    }

    #[test]
    fn lines_split_correctly() {
        let content = "line1\nline2\nline3\n".to_string();
        let doc = Document::from_content(PathBuf::from("test.org"), content);
        assert_eq!(doc.lines, vec!["line1", "line2", "line3"]);
    }
}
