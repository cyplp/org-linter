use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Collect all `.org` files under `path` (or just `path` itself if it's a file).
pub fn collect_org_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        if is_org(path) {
            return vec![path.to_path_buf()];
        }
        return vec![];
    }

    WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|result| result.ok())
        .filter(|entry| entry.file_type().is_file() && is_org(entry.path()))
        .map(|entry| entry.into_path())
        .collect()
}

fn is_org(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("org")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn single_file() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("notes.org");
        fs::write(&file_path, "* test\n").unwrap();
        let files = collect_org_files(&file_path);
        assert_eq!(files, vec![file_path]);
    }

    #[test]
    fn ignores_non_org_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.md"), "# md").unwrap();
        fs::write(dir.path().join("notes.org"), "* org").unwrap();
        let files = collect_org_files(dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].to_str().unwrap().ends_with(".org"));
    }

    #[test]
    fn walks_subdirectories() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("deep.org"), "* H\n").unwrap();
        let files = collect_org_files(dir.path());
        assert_eq!(files.len(), 1);
    }
}
