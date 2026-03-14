use std::{path::Path, process::Command};

use gtk::glib::markup_escape_text;

pub(super) fn render_docx_html(path: &Path) -> Result<String, String> {
    let xml = extract_document_xml(path)?;
    let paragraphs = parse_paragraphs(&xml);
    if paragraphs.is_empty() {
        return Err("document has no readable text".to_string());
    }

    Ok(build_html(&paragraphs))
}

fn extract_document_xml(path: &Path) -> Result<String, String> {
    let output = Command::new("unzip")
        .args(["-p", &path.display().to_string(), "word/document.xml"])
        .output()
        .map_err(|error| format!("docx preview unavailable: {error}"))?;

    if !output.status.success() {
        return Err("unable to extract document.xml".to_string());
    }

    String::from_utf8(output.stdout).map_err(|_| "document.xml is not valid utf-8".to_string())
}

fn parse_paragraphs(xml: &str) -> Vec<String> {
    xml.split("<w:p")
        .skip(1)
        .filter_map(paragraph_text)
        .filter(|text| !text.trim().is_empty())
        .collect()
}

fn paragraph_text(chunk: &str) -> Option<String> {
    let end = chunk.find("</w:p>")?;
    let paragraph = &chunk[..end];
    let mut text = String::new();
    let mut cursor = paragraph;

    while let Some(start) = cursor.find("<w:t") {
        let after_tag = &cursor[start..];
        let open_end = after_tag.find('>')?;
        let after_open = &after_tag[open_end + 1..];
        let close = after_open.find("</w:t>")?;
        text.push_str(&decode_xml_entities(&after_open[..close]));
        cursor = &after_open[close + "</w:t>".len()..];
    }

    if text.is_empty() {
        return None;
    }

    Some(text)
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn build_html(paragraphs: &[String]) -> String {
    let mut body = String::new();
    for paragraph in paragraphs {
        body.push_str("<p>");
        body.push_str(&markup_escape_text(paragraph));
        body.push_str("</p>");
    }

    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><style>{}</style></head><body>{}</body></html>",
        "html,body{background:#0b0b0b;color:#f3efe5;font-family:'DejaVu Sans Mono',monospace;margin:0;}body{padding:20px 18px 28px;line-height:1.7;font-size:13px;}p{margin:0 0 12px;white-space:pre-wrap;}::selection{background:#ff4d4d;color:#0b0b0b;}",
        body
    )
}
