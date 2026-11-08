//! XML and RSS/Atom feed extraction.

/// Extracted content from XML/RSS/Atom feeds.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtractedXml {
    pub content_type: XmlContentType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub items: Vec<FeedItem>,
    pub raw_text: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum XmlContentType {
    Rss,
    Atom,
    GenericXml,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedItem {
    pub title: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub published: Option<String>,
}

/// Extract content from XML text.
/// Uses simple regex-based extraction for RSS/Atom feeds.
pub fn extract_xml(text: &str) -> ExtractedXml {
    let content_type = detect_xml_type(text);

    match content_type {
        XmlContentType::Rss => extract_rss(text),
        XmlContentType::Atom => extract_atom(text),
        XmlContentType::GenericXml => ExtractedXml {
            content_type: XmlContentType::GenericXml,
            title: None,
            description: None,
            items: Vec::new(),
            raw_text: if text.len() > 10_000 { text[..10_000].to_string() } else { text.to_string() },
        },
    }
}

fn detect_xml_type(text: &str) -> XmlContentType {
    let lower = text.to_lowercase();
    if lower.contains("<rss") || lower.contains("<channel>") {
        XmlContentType::Rss
    } else if lower.contains("<feed") && lower.contains("xmlns=\"http://www.w3.org/2005/atom\"") {
        XmlContentType::Atom
    } else {
        XmlContentType::GenericXml
    }
}

fn extract_rss(text: &str) -> ExtractedXml {
    let title = extract_tag_content(text, "title");
    let description = extract_tag_content(text, "description");

    let mut items = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("<item") {
        if let Some(end) = remaining[start..].find("</item>") {
            let item_text = &remaining[start..start + end + 7];
            items.push(FeedItem {
                title: extract_tag_content(item_text, "title").unwrap_or_default(),
                link: extract_tag_content(item_text, "link"),
                description: extract_tag_content(item_text, "description"),
                published: extract_tag_content(item_text, "pubDate"),
            });
            remaining = &remaining[start + end + 7..];
        } else {
            break;
        }
    }

    ExtractedXml {
        content_type: XmlContentType::Rss,
        title,
        description,
        items,
        raw_text: String::new(),
    }
}

fn extract_atom(text: &str) -> ExtractedXml {
    let title = extract_tag_content(text, "title");

    let mut items = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("<entry") {
        if let Some(end) = remaining[start..].find("</entry>") {
            let entry_text = &remaining[start..start + end + 8];
            let link = extract_attr_value(entry_text, "link", "href");
            items.push(FeedItem {
                title: extract_tag_content(entry_text, "title").unwrap_or_default(),
                link,
                description: extract_tag_content(entry_text, "summary")
                    .or_else(|| extract_tag_content(entry_text, "content")),
                published: extract_tag_content(entry_text, "published")
                    .or_else(|| extract_tag_content(entry_text, "updated")),
            });
            remaining = &remaining[start + end + 8..];
        } else {
            break;
        }
    }

    ExtractedXml {
        content_type: XmlContentType::Atom,
        title,
        description: None,
        items,
        raw_text: String::new(),
    }
}

fn extract_tag_content(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    if let Some(start) = text.find(&open) {
        let after_open = &text[start..];
        if let Some(gt) = after_open.find('>') {
            let content_start = start + gt + 1;
            let close = format!("</{}>", tag);
            if let Some(end) = text[content_start..].find(&close) {
                let content = text[content_start..content_start + end].trim();
                // Strip CDATA
                let content = if content.starts_with("<![CDATA[") && content.ends_with("]]>") {
                    &content[9..content.len() - 3]
                } else {
                    content
                };
                if !content.is_empty() {
                    return Some(content.to_string());
                }
            }
        }
    }
    None
}

fn extract_attr_value(text: &str, tag: &str, attr: &str) -> Option<String> {
    let open = format!("<{}", tag);
    if let Some(start) = text.find(&open) {
        let tag_text = &text[start..];
        let end = tag_text.find('>').unwrap_or(tag_text.len());
        let tag_str = &tag_text[..end];
        let attr_pattern = format!("{}=\"", attr);
        if let Some(attr_start) = tag_str.find(&attr_pattern) {
            let val_start = attr_start + attr_pattern.len();
            if let Some(val_end) = tag_str[val_start..].find('"') {
                return Some(tag_str[val_start..val_start + val_end].to_string());
            }
        }
    }
    None
}
