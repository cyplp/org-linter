use crate::{document::Document, violation::Violation};
use super::{Rule, Fixable};
use std::time::Duration;

fn is_in_code_block(doc_lines: &[String], line_idx: usize) -> bool {
    let mut in_block = false;
    for (index, line) in doc_lines.iter().enumerate() {
        if index > line_idx { break; }
        let trimmed = line.trim().to_uppercase();
        if trimmed.starts_with("#+BEGIN_SRC") || trimmed.starts_with("#+BEGIN_EXAMPLE") {
            in_block = true;
        } else if trimmed.starts_with("#+END_SRC") || trimmed.starts_with("#+END_EXAMPLE") {
            in_block = false;
        }
    }
    in_block
}

fn url_end(text: &str) -> usize {
    text.find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | '>' | '<' | ')' | ',' | ']'))
        .unwrap_or(text.len())
}

fn bare_url_ranges(line: &str) -> Vec<(usize, usize)> {
    let mut result = Vec::new();
    let mut pos = 0;
    while pos < line.len() {
        if line[pos..].starts_with("[[") {
            if let Some(closing_offset) = line[pos + 2..].find("]]") {
                pos = pos + 2 + closing_offset + 2;
                continue;
            }
        }
        if line[pos..].starts_with("http://") || line[pos..].starts_with("https://") {
            let len = url_end(&line[pos..]);
            result.push((pos, pos + len));
            pos += len;
            continue;
        }
        pos += line[pos..].chars().next().map(|ch| ch.len_utf8()).unwrap_or(1);
    }
    result
}

fn domain_from_url(url: &str) -> String {
    let without_scheme = url.trim_start_matches("https://").trim_start_matches("http://");
    without_scheme.split(|ch| matches!(ch, '/' | '?' | '#')).next().unwrap_or(without_scheme).to_string()
}

fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let tag_start = lower.find("<title")?;
    let content_start = lower[tag_start..].find('>')? + tag_start + 1;
    let content_end = lower[content_start..].find("</title>")? + content_start;
    let title = html[content_start..content_end].trim();
    if title.is_empty() { None } else { Some(decode_entities(title)) }
}

fn fetch_description(url: &str, client: &reqwest::blocking::Client, timeout: Duration) -> String {
    client.get(url)
        .timeout(timeout)
        .send()
        .ok()
        .filter(|resp| resp.status().is_success())
        .and_then(|resp| resp.text().ok())
        .and_then(|html| extract_title(&html))
        .unwrap_or_else(|| domain_from_url(url))
}

fn fix_line(
    line: &str,
    client: &reqwest::blocking::Client,
    timeout: Duration,
    cache: &mut std::collections::HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut pos = 0;
    while pos < line.len() {
        if line[pos..].starts_with("[[") {
            if let Some(closing_offset) = line[pos + 2..].find("]]") {
                let end = pos + 2 + closing_offset + 2;
                result.push_str(&line[pos..end]);
                pos = end;
                continue;
            }
        }
        if line[pos..].starts_with("http://") || line[pos..].starts_with("https://") {
            let len = url_end(&line[pos..]);
            let url = &line[pos..pos + len];
            let desc = cache.entry(url.to_string())
                .or_insert_with(|| fetch_description(url, client, timeout))
                .clone();
            let org_url = url.replacen("://", ":", 1);
            result.push_str(&format!("[[{}][{}]]", org_url, desc));
            pos += len;
            continue;
        }
        let ch = line[pos..].chars().next().unwrap();
        result.push(ch);
        pos += ch.len_utf8();
    }
    result
}

pub struct BareUrl {
    pub timeout_secs: u64,
}

impl Default for BareUrl {
    fn default() -> Self { Self { timeout_secs: 3 } }
}

impl Rule for BareUrl {
    fn id(&self) -> &str { "ORG013" }
    fn description(&self) -> &str { "HTTP/HTTPS URLs must be wrapped in org-mode markup links" }

    fn check(&self, doc: &Document) -> Vec<Violation> {
        doc.lines.iter().enumerate().flat_map(|(line_idx, line)| {
            if is_in_code_block(&doc.lines, line_idx) {
                return vec![];
            }
            bare_url_ranges(line).into_iter().map(move |(start, end)| {
                Violation::new("ORG013", line_idx + 1, format!("Bare URL: {}", &line[start..end]))
            }).collect::<Vec<_>>()
        }).collect()
    }
}

impl Fixable for BareUrl {
    fn fix(&self, doc: &mut Document) -> usize {
        let timeout = Duration::from_secs(self.timeout_secs);
        let client = reqwest::blocking::Client::new();
        let mut cache = std::collections::HashMap::new();
        let mut count = 0;
        for (line_idx, line) in doc.lines.iter_mut().enumerate() {
            if is_in_code_block(&doc.lines, line_idx) { continue; }
            if bare_url_ranges(line).is_empty() { continue; }
            let fixed = fix_line(line, &client, timeout, &mut cache);
            if fixed != *line {
                *line = fixed;
                count += 1;
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
    fn no_violations_when_all_linked() {
        let document = doc("See [[https://example.com][Example]] for details.\n");
        assert!(BareUrl::default().check(&document).is_empty());
    }

    #[test]
    fn bare_http_detected() {
        let document = doc("Visit http://example.com today.\n");
        let violations = BareUrl::default().check(&document);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, Some(1));
        assert!(violations[0].message.contains("http://example.com"));
    }

    #[test]
    fn bare_https_detected() {
        let document = doc("See https://example.com for more.\n");
        let violations = BareUrl::default().check(&document);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].message.contains("https://example.com"));
    }

    #[test]
    fn multiple_bare_urls_same_line() {
        let document = doc("Go to https://a.com and https://b.com.\n");
        assert_eq!(BareUrl::default().check(&document).len(), 2);
    }

    #[test]
    fn linked_url_not_detected() {
        let document = doc("[[https://example.com][Example site]]\n");
        assert!(BareUrl::default().check(&document).is_empty());
    }

    #[test]
    fn url_only_link_not_detected() {
        let document = doc("[[https://example.com]]\n");
        assert!(BareUrl::default().check(&document).is_empty());
    }

    #[test]
    fn domain_from_url_strips_scheme_and_path() {
        assert_eq!(domain_from_url("https://www.example.com/path/to/page"), "www.example.com");
        assert_eq!(domain_from_url("http://example.com"), "example.com");
    }

    #[test]
    fn domain_from_url_with_query_and_fragment() {
        assert_eq!(domain_from_url("https://example.com?foo=bar"), "example.com");
        assert_eq!(domain_from_url("https://example.com#section"), "example.com");
    }

    #[test]
    fn extract_title_basic() {
        let html = "<html><head><title>Hello World</title></head></html>";
        assert_eq!(extract_title(html), Some("Hello World".to_string()));
    }

    #[test]
    fn extract_title_with_tag_attrs() {
        let html = "<html><head><title lang=\"en\">My Page</title></head></html>";
        assert_eq!(extract_title(html), Some("My Page".to_string()));
    }

    #[test]
    fn extract_title_empty_returns_none() {
        let html = "<html><head><title></title></head></html>";
        assert_eq!(extract_title(html), None);
    }

    #[test]
    fn extract_title_missing_returns_none() {
        let html = "<html><head></head><body>No title here</body></html>";
        assert_eq!(extract_title(html), None);
    }

    #[test]
    fn extract_title_decodes_entities() {
        let html = "<html><head><title>Tom &amp; Jerry</title></head></html>";
        assert_eq!(extract_title(html), Some("Tom & Jerry".to_string()));
    }

    #[test]
    fn fix_falls_back_to_domain_on_failure() {
        // .invalid TLD is guaranteed non-resolvable (RFC 2606)
        let mut document = doc("See https://this.does.not.exist.invalid/ for info.\n");
        BareUrl { timeout_secs: 3 }.fix(&mut document);
        assert_eq!(
            document.lines[0],
            "See [[https:this.does.not.exist.invalid/][this.does.not.exist.invalid]] for info."
        );
    }

    #[test]
    fn url_in_code_block_not_detected() {
        let document = doc("#+BEGIN_SRC rust\nhttps://example.com\n#+END_SRC\n");
        assert!(BareUrl::default().check(&document).is_empty());
    }

    #[test]
    fn url_in_example_block_not_detected() {
        let document = doc("#+BEGIN_EXAMPLE\nhttps://example.com\n#+END_EXAMPLE\n");
        assert!(BareUrl::default().check(&document).is_empty());
    }

    #[test]
    fn url_outside_code_block_detected() {
        let document = doc("https://example.com\n#+BEGIN_SRC rust\n#+END_SRC\n");
        assert_eq!(BareUrl::default().check(&document).len(), 1);
    }
}
