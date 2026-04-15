use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use html2text::from_read;
use quick_xml::{Reader, events::Event};
use serde::Deserialize;

use crate::{
    i18n::{Language, tr},
    paths,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentKind {
    Xml,
    Pdf,
}

#[derive(Clone, Debug)]
pub struct DocumentData {
    pub title: String,
    pub source_path: PathBuf,
    pub kind: DocumentKind,
    pub preview_html: Option<String>,
    pub preview_text: String,
    pub secondary_text: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct HelperResponse {
    title: Option<String>,
    html: Option<String>,
    xml: Option<String>,
    warnings: Option<Vec<String>>,
}

pub fn load_document(path: &Path, language: Language) -> Result<DocumentData> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());

    match extension.as_deref() {
        Some("xml") => load_xml_document(path, language),
        Some("pdf") => load_pdf_document(path, language),
        _ => bail!("{}: {}", tr(language, "unsupported-file"), path.display()),
    }
}

fn load_xml_document(path: &Path, language: Language) -> Result<DocumentData> {
    let raw_xml = fs::read_to_string(path)
        .with_context(|| format!("{} {}", tr(language, "read-xml-failed"), path.display()))?;
    let root = detect_xml_root(&raw_xml).unwrap_or_else(|| tr(language, "unknown-xml").to_string());

    let helper = run_helper(language, "render-xml", path).ok();
    let warnings = helper
        .as_ref()
        .and_then(|response| response.warnings.clone())
        .unwrap_or_default();

    let preview_text = helper
        .as_ref()
        .and_then(|response| response.html.as_ref())
        .map(|html| html_to_text(html))
        .filter(|text| !text.trim().is_empty())
        .unwrap_or_else(|| fallback_xml_preview(language, &raw_xml, &root));

    Ok(DocumentData {
        title: helper
            .as_ref()
            .and_then(|response| response.title.clone())
            .unwrap_or_else(|| paths::display_path(path)),
        source_path: path.to_path_buf(),
        kind: DocumentKind::Xml,
        preview_html: helper.and_then(|response| response.html),
        preview_text,
        secondary_text: raw_xml,
        warnings,
    })
}

fn load_pdf_document(path: &Path, language: Language) -> Result<DocumentData> {
    let helper = run_helper(language, "render-pdf", path).ok();
    let title = helper
        .as_ref()
        .and_then(|response| response.title.clone())
        .unwrap_or_else(|| paths::display_path(path));

    let warnings = helper
        .as_ref()
        .and_then(|response| response.warnings.clone())
        .unwrap_or_default();

    if let Some(response) = helper {
        if let Some(xml) = response.xml {
            let preview_text = response
                .html
                .as_deref()
                .map(html_to_text)
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| {
                    fallback_xml_preview(language, &xml, tr(language, "embedded-invoice-xml"))
                });

            return Ok(DocumentData {
                title,
                source_path: path.to_path_buf(),
                kind: DocumentKind::Pdf,
                preview_html: response.html,
                preview_text,
                secondary_text: xml,
                warnings,
            });
        }
    }

    let preview_text = [
        tr(language, "pdf-no-embedded-xml-1"),
        "",
        tr(language, "pdf-no-embedded-xml-2"),
        tr(language, "pdf-no-embedded-xml-3"),
    ]
    .join("\n");

    Ok(DocumentData {
        title,
        source_path: path.to_path_buf(),
        kind: DocumentKind::Pdf,
        preview_html: None,
        preview_text,
        secondary_text: tr(language, "source-pdf-missing").replace("{}", &path.display().to_string()),
        warnings,
    })
}

fn run_helper(language: Language, command: &str, path: &Path) -> Result<HelperResponse> {
    if !paths::helper_script().exists() || !paths::has_legacy_assets() {
        bail!("{}", tr(language, "helper-assets-missing"));
    }

    let output = Command::new(paths::node_binary())
        .arg(paths::helper_script())
        .arg(command)
        .arg(path)
        .current_dir(paths::project_root())
        .output()
        .with_context(|| {
            tr(language, "helper-launch-failed").replace("{}", &path.display().to_string())
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("{}", tr(language, "helper-failed").replace("{}", &stderr));
    }

    let stdout = String::from_utf8(output.stdout).context(tr(language, "helper-utf8-failed"))?;
    serde_json::from_str(&stdout).context(tr(language, "helper-json-failed"))
}

fn fallback_xml_preview(language: Language, xml: &str, root: &str) -> String {
    let lines = xml.lines().count();
    tr(language, "fallback-preview")
        .replace("{root}", root)
        .replace("{lines}", &lines.to_string())
}

fn html_to_text(html: &str) -> String {
    from_read(html.as_bytes(), 100).unwrap_or_else(|_| html.to_string())
}

fn detect_xml_root(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                return Some(String::from_utf8_lossy(start.name().as_ref()).to_string());
            }
            Ok(Event::Eof) => return None,
            Ok(_) => {}
            Err(_) => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{detect_xml_root, html_to_text};

    #[test]
    fn detects_root_element() {
        let xml = r#"<?xml version="1.0"?><Invoice><cbc:ID>1</cbc:ID></Invoice>"#;
        assert_eq!(detect_xml_root(xml).as_deref(), Some("Invoice"));
    }

    #[test]
    fn strips_html_to_text() {
        let text = html_to_text("<h1>Invoice</h1><p>Total: 42</p>");
        assert!(text.contains("Invoice"));
        assert!(text.contains("Total: 42"));
    }
}
