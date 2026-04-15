#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    English,
    Romanian,
}

pub fn detect_language() -> Language {
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_ascii_lowercase();
    if locale.starts_with("ro") {
        Language::Romanian
    } else {
        Language::English
    }
}

pub fn tr(language: Language, key: &'static str) -> &'static str {
    match (language, key) {
        (Language::English, "app-title") => "Quba GTK",
        (Language::Romanian, "app-title") => "Quba GTK",
        (Language::English, "app-subtitle") => "Native GTK invoice viewer",
        (Language::Romanian, "app-subtitle") => "Vizualizator nativ GTK pentru facturi",
        (Language::English, "open-tooltip") => "Open a bill or hybrid invoice",
        (Language::Romanian, "open-tooltip") => "Deschide o factura sau o factura hibrida",
        (Language::English, "examples-tooltip") => "Open example invoices website",
        (Language::Romanian, "examples-tooltip") => "Deschide pagina cu exemple de facturi",
        (Language::English, "about-tooltip") => "About Quba GTK",
        (Language::Romanian, "about-tooltip") => "Despre Quba GTK",
        (Language::English, "welcome-body") => {
            "Open XML bills and hybrid invoice PDFs from the file chooser, command line, or file manager."
        }
        (Language::Romanian, "welcome-body") => {
            "Deschide facturi XML si PDF-uri hibride din selectorul de fisiere, linia de comanda sau managerul de fisiere."
        }
        (Language::English, "status-nonlocal") => {
            "A non-local file was ignored because Quba GTK currently supports local files only."
        }
        (Language::Romanian, "status-nonlocal") => {
            "Un fisier nelocal a fost ignorat deoarece Quba GTK suporta momentan doar fisiere locale."
        }
        (Language::English, "open-dialog-title") => "Open Invoice",
        (Language::Romanian, "open-dialog-title") => "Deschide factura",
        (Language::English, "open-dialog-filter") => "Invoices and XML/PDF documents",
        (Language::Romanian, "open-dialog-filter") => "Facturi si documente XML/PDF",
        (Language::English, "status-opened") => "Opened",
        (Language::Romanian, "status-opened") => "Deschis",
        (Language::English, "status-opened-warning") => "warning(s)",
        (Language::Romanian, "status-opened-warning") => "avertisment(e)",
        (Language::English, "error-open") => "Unable to open document",
        (Language::Romanian, "error-open") => "Documentul nu a putut fi deschis",
        (Language::English, "banner-xml") => "Native XML invoice view",
        (Language::Romanian, "banner-xml") => "Vizualizare nativa pentru factura XML",
        (Language::English, "banner-pdf") => "Hybrid PDF view",
        (Language::Romanian, "banner-pdf") => "Vizualizare PDF hibrid",
        (Language::English, "warnings") => "Warnings",
        (Language::Romanian, "warnings") => "Avertismente",
        (Language::English, "structured-preview") => "Structured Preview",
        (Language::Romanian, "structured-preview") => "Previzualizare structurata",
        (Language::English, "structured-tab") => "Structured",
        (Language::Romanian, "structured-tab") => "Structurat",
        (Language::English, "pdf-tab") => "PDF",
        (Language::Romanian, "pdf-tab") => "PDF",
        (Language::English, "source-tab") => "Source",
        (Language::Romanian, "source-tab") => "Sursa",
        (Language::English, "search-tooltip") => "Search in the current view",
        (Language::Romanian, "search-tooltip") => "Cauta in vizualizarea curenta",
        (Language::English, "print-tooltip") => "Print the current web view",
        (Language::Romanian, "print-tooltip") => "Tipareste vizualizarea web curenta",
        (Language::English, "search-placeholder") => "Search document",
        (Language::Romanian, "search-placeholder") => "Cauta in document",
        (Language::English, "source-xml") => "Source XML",
        (Language::Romanian, "source-xml") => "XML sursa",
        (Language::English, "raw-details") => "Embedded or Raw Details",
        (Language::Romanian, "raw-details") => "Detalii incorporate sau brute",
        (Language::English, "status-external-failed") => "Failed to open external viewer",
        (Language::Romanian, "status-external-failed") => "Deschiderea vizualizatorului extern a esuat",
        (Language::English, "about-heading") => "About Quba GTK",
        (Language::Romanian, "about-heading") => "Despre Quba GTK",
        (Language::English, "about-body") => "Native GTK4/libadwaita rewrite of Quba for XML bills and hybrid invoice PDFs.",
        (Language::Romanian, "about-body") => "Rescriere nativa GTK4/libadwaita a aplicatiei Quba pentru facturi XML si PDF-uri hibride.",
        (Language::English, "about-developer") => "stf_ftw's rewrite workspace",
        (Language::Romanian, "about-developer") => "Spatiul de lucru pentru rescrierea realizata de stf_ftw",
        (Language::English, "details-ai") => "This application includes AI-generated code made with OpenAI's Codex, reveiwed and adapted for this rewrite.",
        (Language::Romanian, "details-ai") => "Aceasta aplicatie include cod generat cu ajutorul AI-ului OpenAI Codex, apoi revizuit si adaptat pentru aceasta rescriere.",
        (Language::English, "details-engine") => "Quba GTK replaces the original Electron shell with a native GTK4/libadwaita interface while reusing Quba's invoice transformation pipeline, including its XSLT and helper-based rendering engine.",
        (Language::Romanian, "details-engine") => "Quba GTK inlocuieste shell-ul original Electron cu o interfata nativa GTK4/libadwaita, dar reutilizeaza pipeline-ul Quba de transformare a facturilor, inclusiv motorul de randare bazat pe XSLT si helper.",
        (Language::English, "upstream-website") => "Upstream website",
        (Language::Romanian, "upstream-website") => "Site-ul proiectului original",
        (Language::English, "unsupported-file") => "Unsupported file type",
        (Language::Romanian, "unsupported-file") => "Tip de fisier nesuportat",
        (Language::English, "read-xml-failed") => "Failed to read XML file",
        (Language::Romanian, "read-xml-failed") => "Citirea fisierului XML a esuat",
        (Language::English, "unknown-xml") => "Unknown XML",
        (Language::Romanian, "unknown-xml") => "XML necunoscut",
        (Language::English, "embedded-invoice-xml") => "Embedded invoice XML",
        (Language::Romanian, "embedded-invoice-xml") => "XML de factura incorporat",
        (Language::English, "pdf-no-embedded-xml-1") => "No embedded XML invoice was extracted from this PDF.",
        (Language::Romanian, "pdf-no-embedded-xml-1") => "Nu a fost extras niciun XML de factura incorporat din acest PDF.",
        (Language::English, "pdf-no-embedded-xml-2") => "Quba GTK can still hand the PDF off to your default PDF viewer.",
        (Language::Romanian, "pdf-no-embedded-xml-2") => "Quba GTK poate totusi sa trimita PDF-ul catre vizualizatorul PDF implicit.",
        (Language::English, "pdf-no-embedded-xml-3") => "If this is a Factur-X/ZUGFeRD or XRechnung hybrid invoice, make sure the helper dependencies are installed so embedded XML extraction is available.",
        (Language::Romanian, "pdf-no-embedded-xml-3") => "Daca acesta este un hibrid Factur-X/ZUGFeRD sau XRechnung, asigura-te ca dependintele helper-ului sunt instalate pentru a permite extragerea XML-ului incorporat.",
        (Language::English, "source-pdf-missing") => "Source PDF: {}\n\nEmbedded XML was not detected or could not be rendered.",
        (Language::Romanian, "source-pdf-missing") => "PDF sursa: {}\n\nXML-ul incorporat nu a fost detectat sau nu a putut fi randat.",
        (Language::English, "helper-assets-missing") => "helper assets are not available",
        (Language::Romanian, "helper-assets-missing") => "resursele helper nu sunt disponibile",
        (Language::English, "helper-launch-failed") => "Failed to launch helper for {}",
        (Language::Romanian, "helper-launch-failed") => "Pornirea helper-ului pentru {} a esuat",
        (Language::English, "helper-failed") => "helper failed: {}",
        (Language::Romanian, "helper-failed") => "helper-ul a esuat: {}",
        (Language::English, "helper-utf8-failed") => "helper output was not valid UTF-8",
        (Language::Romanian, "helper-utf8-failed") => "iesirea helper-ului nu a fost UTF-8 valid",
        (Language::English, "helper-json-failed") => "helper output was not valid JSON",
        (Language::Romanian, "helper-json-failed") => "iesirea helper-ului nu a fost JSON valid",
        (Language::English, "fallback-preview") => "{root}\n\nA native text preview is shown because HTML rendering is not available yet.\n\nFile has {lines} lines.",
        (Language::Romanian, "fallback-preview") => "{root}\n\nEste afisata o previzualizare text nativa deoarece randarea HTML nu este disponibila inca.\n\nFisierul are {lines} linii.",
        _ => key,
    }
}
