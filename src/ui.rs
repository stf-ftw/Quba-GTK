use std::{cell::RefCell, path::Path, rc::Rc};

use adw::prelude::*;
use gtk::{Align, Orientation, WrapMode, gdk, gio, glib};
use sourceview5 as sourceview;
use sourceview5::prelude::*;
use webkit6::{FindOptions, PrintOperation, WebView};
use webkit6::prelude::*;

use crate::{
    document::{DocumentData, DocumentKind, load_document},
    i18n::{Language, detect_language, tr},
};

#[derive(Clone)]
pub struct AppState {
    pub window: adw::ApplicationWindow,
    pub tab_view: adw::TabView,
    pub status_label: gtk::Label,
    pub language: Language,
}

thread_local! {
    static APP_STATE: RefCell<Option<Rc<AppState>>> = const { RefCell::new(None) };
}

pub fn build_startup(app: &adw::Application) {
    let open_action = gio::SimpleAction::new("open-file", None);
    let app_weak = app.downgrade();
    open_action.connect_activate(move |_, _| {
        if let Some(app) = app_weak.upgrade() {
            open_file_dialog(&app);
        }
    });
    app.add_action(&open_action);
    app.set_accels_for_action("app.open-file", &["<Primary>o"]);
}

pub fn build_ui(app: &adw::Application) {
    let state = ensure_state(app);
    state.window.present();
}

pub fn open_files(app: &adw::Application, files: &[gio::File], _hint: &str) {
    let state = ensure_state(app);
    for file in files {
        if let Some(path) = file.path() {
            add_document_tab(&state, &path);
        } else {
            let language = state.language;
            state.status_label.set_text(tr(language, "status-nonlocal"));
        }
    }
    state.window.present();
}

fn ensure_state(app: &adw::Application) -> Rc<AppState> {
    if let Some(state) = APP_STATE.with(|state| state.borrow().clone()) {
        return state;
    }

    let language = detect_language();

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .default_width(1200)
        .default_height(760)
        .build();

    let header = adw::HeaderBar::new();
    let title = adw::WindowTitle::builder()
        .title(tr(language, "app-title"))
        .subtitle(tr(language, "app-subtitle"))
        .build();
    header.set_title_widget(Some(&title));

    let open_button = gtk::Button::builder()
        .icon_name("document-open-symbolic")
        .tooltip_text(tr(language, "open-tooltip"))
        .build();
    let app_weak = app.downgrade();
    open_button.connect_clicked(move |_| {
        if let Some(app) = app_weak.upgrade() {
            open_file_dialog(&app);
        }
    });
    header.pack_start(&open_button);

    let examples_button = gtk::Button::builder()
        .icon_name("web-browser-symbolic")
        .tooltip_text(tr(language, "examples-tooltip"))
        .build();
    examples_button.connect_clicked(|_| {
        let _ = gio::AppInfo::launch_default_for_uri(
            "https://quba-viewer.org/beispiele/?pk_campaign=examples&pk_source=application",
            gio::AppLaunchContext::NONE,
        );
    });
    header.pack_start(&examples_button);

    let about_button = gtk::Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text(tr(language, "about-tooltip"))
        .build();
    let window_weak = window.downgrade();
    about_button.connect_clicked(move |_| {
        if let Some(window) = window_weak.upgrade() {
            let dialog = adw::AboutDialog::builder()
                .application_name("Quba GTK")
                .application_icon("org.zugferd.QubaViewer")
                .developer_name(tr(language, "about-developer"))
                .version(env!("CARGO_PKG_VERSION"))
                .comments(&format!(
                    "{}\n\n{}\n\n{}",
                    tr(language, "about-body"),
                    tr(language, "details-ai"),
                    tr(language, "details-engine")
                ))
                .issue_url("https://github.com/stf-ftw/Quba-GTK")
                .build();
            dialog.add_link(tr(language, "upstream-website"), "https://quba-viewer.org");
            dialog.present(Some(&window));
        }
    });
    header.pack_end(&about_button);

    let status_label = gtk::Label::builder()
        .xalign(0.0)
        .wrap(true)
        .css_classes(["dim-label"])
        .build();

    let tab_view = adw::TabView::new();
    let tab_bar = adw::TabBar::builder().view(&tab_view).build();

    let welcome_bin = build_welcome(language);

    let stack = gtk::Stack::new();
    stack.add_named(&welcome_bin, Some("welcome"));
    stack.add_named(&tab_view, Some("tabs"));
    stack.set_visible_child_name("welcome");

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header);
    toolbar_view.add_top_bar(&tab_bar);
    toolbar_view.set_content(Some(&stack));

    let status_bar = gtk::ActionBar::new();
    status_bar.set_revealed(true);
    status_bar.pack_start(&status_label);
    toolbar_view.add_bottom_bar(&status_bar);

    window.set_content(Some(&toolbar_view));

    let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
    let window_weak = window.downgrade();
    drop_target.connect_drop(move |_, value, _, _| {
        let Some(window) = window_weak.upgrade() else {
            return false;
        };
        let Ok(file_list) = value.get::<gdk::FileList>() else {
            return false;
        };

        if let Some(app) = window.application().and_downcast::<adw::Application>() {
            for file in file_list.files() {
                open_files(&app, &[file], "");
            }
            true
        } else {
            false
        }
    });
    window.add_controller(drop_target);

    let state = Rc::new(AppState {
        window,
        tab_view: tab_view.clone(),
        status_label,
        language,
    });

    let stack_clone = stack.clone();
    tab_view.connect_n_pages_notify(move |view| {
        let visible = if view.n_pages() == 0 { "welcome" } else { "tabs" };
        stack_clone.set_visible_child_name(visible);
    });

    APP_STATE.with(|slot| {
        *slot.borrow_mut() = Some(state.clone());
    });

    state
}

fn open_file_dialog(app: &adw::Application) {
    let state = ensure_state(app);
    let language = state.language;
    let dialog = gtk::FileDialog::builder()
        .title(tr(language, "open-dialog-title"))
        .build();

    let filter = gtk::FileFilter::new();
    filter.add_mime_type("application/pdf");
    filter.add_pattern("*.xml");
    filter.set_name(Some(tr(language, "open-dialog-filter")));
    let filters = gio::ListStore::new::<gtk::FileFilter>();
    filters.append(&filter);
    dialog.set_filters(Some(&filters));
    dialog.set_default_filter(Some(&filter));

    dialog.open_multiple(
        Some(&state.window),
        gio::Cancellable::NONE,
        glib::clone!(
            #[strong]
            state,
            move |result| match result {
                Ok(model) => {
                    for index in 0..model.n_items() {
                        if let Some(obj) = model.item(index) {
                            if let Ok(file) = obj.downcast::<gio::File>() {
                                if let Some(path) = file.path() {
                                    add_document_tab(&state, &path);
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    if err.matches(gio::IOErrorEnum::Cancelled) {
                        return;
                    }
                    state.status_label.set_text(&err.to_string());
                }
            }
        ),
    );
}

fn add_document_tab(state: &Rc<AppState>, path: &Path) {
    let language = state.language;
    match load_document(path, language) {
        Ok(document) => {
            let widget = build_document_page(&document, &state.status_label, language);
            let page = state.tab_view.append(&widget);
            page.set_title(&document.title);
            page.set_tooltip(&document.source_path.display().to_string());
            page.set_indicator_icon(Some(&icon_for_kind(document.kind.clone())));
            state.tab_view.set_selected_page(&page);

            let status = if document.warnings.is_empty() {
                format!("{} {}", tr(language, "status-opened"), document.source_path.display())
            } else {
                format!(
                    "{} {} with {} {}",
                    tr(language, "status-opened"),
                    document.source_path.display(),
                    document.warnings.len()
                    ,
                    tr(language, "status-opened-warning")
                )
            };
            state.status_label.set_text(&status);
        }
        Err(err) => {
            state.status_label.set_text(&err.to_string());
            show_error_dialog(&state.window, tr(language, "error-open"), &err.to_string());
        }
    }
}

fn build_document_page(
    document: &DocumentData,
    _status_label: &gtk::Label,
    language: Language,
) -> gtk::Widget {
    let container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .vexpand(true)
        .build();

    let clamp = adw::Clamp::builder().maximum_size(1400).build();
    clamp.set_child(Some(&container));

    let title_group = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(3)
        .build();
    let title = gtk::Label::builder()
        .label(&document.title)
        .xalign(0.0)
        .halign(Align::Start)
        .css_classes(["title-3"])
        .build();
    let subtitle = gtk::Label::builder()
        .label(match document.kind {
            DocumentKind::Xml => tr(language, "banner-xml"),
            DocumentKind::Pdf => tr(language, "banner-pdf"),
        })
        .xalign(0.0)
        .halign(Align::Start)
        .css_classes(["dim-label"])
        .build();
    title_group.append(&title);
    title_group.append(&subtitle);
    container.append(&title_group);

    if !document.warnings.is_empty() {
        let warnings = document.warnings.join("\n");
        let warning_label = gtk::Label::builder()
            .label(format!("{}:\n{warnings}", tr(language, "warnings")))
            .xalign(0.0)
            .wrap(true)
            .css_classes(["warning"])
            .build();
        container.append(&warning_label);
    }

    let summary = build_preview_pane(document);
    let raw_title = match document.kind {
        DocumentKind::Xml => tr(language, "source-xml"),
        DocumentKind::Pdf => tr(language, "raw-details"),
    };
    let raw = build_source_pane(raw_title, &document.secondary_text);

    let stack = adw::ViewStack::new();
    stack.set_vexpand(true);
    stack.set_hexpand(true);
    stack.add_titled(&summary.widget, Some("structured"), tr(language, "structured-tab"));
    let mut printable_webviews = vec![];
    if let Some(webview) = summary.webview.clone() {
        printable_webviews.push(("structured".to_string(), webview));
    }

    if document.kind == DocumentKind::Pdf {
        let pdf_webview = WebView::new();
        let pdf_uri = gio::File::for_path(&document.source_path).uri();
        pdf_webview.load_uri(&pdf_uri);
        stack.add_titled(&pdf_webview, Some("pdf"), tr(language, "pdf-tab"));
        printable_webviews.push(("pdf".to_string(), pdf_webview));
    }

    stack.add_titled(&raw.widget, Some("source"), tr(language, "source-tab"));
    stack.set_visible_child_name("structured");

    let switcher = adw::ViewSwitcher::builder()
        .stack(&stack)
        .policy(adw::ViewSwitcherPolicy::Wide)
        .halign(Align::Start)
        .build();

    let search_button = gtk::ToggleButton::builder()
        .icon_name("system-search-symbolic")
        .tooltip_text(tr(language, "search-tooltip"))
        .build();
    let print_button = gtk::Button::builder()
        .icon_name("printer-symbolic")
        .tooltip_text(tr(language, "print-tooltip"))
        .build();

    let tools = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(6)
        .build();
    tools.append(&search_button);
    tools.append(&print_button);

    let toolbar_row = gtk::CenterBox::new();
    toolbar_row.add_css_class("toolbar");
    toolbar_row.set_start_widget(Some(&switcher));
    toolbar_row.set_end_widget(Some(&tools));
    container.append(&toolbar_row);

    let search_bar = gtk::SearchBar::new();
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some(tr(language, "search-placeholder")));
    search_bar.connect_entry(&search_entry);
    search_bar.set_child(Some(&search_entry));
    search_bar.set_search_mode(false);
    container.append(&search_bar);
    container.append(&stack);

    {
        let search_bar = search_bar.clone();
        let search_entry = search_entry.clone();
        search_button.connect_toggled(move |button| {
            let active = button.is_active();
            search_bar.set_search_mode(active);
            if active {
                search_entry.grab_focus();
            }
        });
    }

    {
        let stack = stack.clone();
        let source_buffer = raw.buffer.clone();
        let source_view = raw.view.clone();
        let structured_webview = summary.webview.clone();
        let printable_webviews = printable_webviews
            .iter()
            .map(|(name, webview)| (name.clone(), webview.clone()))
            .collect::<Vec<_>>();
        search_entry.connect_search_changed(move |entry| {
            let text = entry.text();
            let query = text.trim();
            let visible = stack.visible_child_name().map(|name| name.to_string());

            if let Some(name) = visible.as_deref() {
                if name == "source" {
                    apply_source_search(&source_buffer, &source_view, query);
                    return;
                }

                for (page_name, webview) in &printable_webviews {
                    if page_name == name {
                        if let Some(controller) = webview.find_controller() {
                            if query.is_empty() {
                                controller.search_finish();
                            } else {
                                controller.search(
                                    query,
                                    FindOptions::CASE_INSENSITIVE.bits(),
                                    u32::MAX,
                                );
                            }
                        }
                        return;
                    }
                }
            }

            if let Some(webview) = &structured_webview {
                if let Some(controller) = webview.find_controller() {
                    controller.search_finish();
                }
            }
        });
    }

    {
        let stack = stack.clone();
        let printable_webviews = printable_webviews
            .iter()
            .map(|(name, webview)| (name.clone(), webview.clone()))
            .collect::<Vec<_>>();
        print_button.connect_clicked(move |_| {
            if let Some(name) = stack.visible_child_name().map(|name| name.to_string()) {
                for (page_name, webview) in &printable_webviews {
                    if *page_name == name {
                        let operation = PrintOperation::new(webview);
                        let _ = operation.run_dialog(None::<&gtk::Window>);
                        return;
                    }
                }
            }
        });
    }

    clamp.upcast()
}

struct PreviewPane {
    widget: gtk::Widget,
    webview: Option<WebView>,
}

struct SourcePane {
    widget: gtk::Widget,
    buffer: sourceview::Buffer,
    view: sourceview::View,
}

fn build_preview_pane(document: &DocumentData) -> PreviewPane {
    if let Some(html) = &document.preview_html {
        let webview = WebView::new();
        webview.load_html(html, Some("file:///"));
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .min_content_height(420)
            .child(&webview)
            .build();
        PreviewPane {
            widget: scroller.upcast(),
            webview: Some(webview),
        }
    } else {
        PreviewPane {
            widget: build_text_scroller(&document.preview_text, true),
            webview: None,
        }
    }
}

fn build_source_pane(_title: &str, content: &str) -> SourcePane {
    let buffer = sourceview::Buffer::new(None);
    buffer.set_text(content);
    buffer.set_highlight_syntax(true);

    let view = sourceview::View::builder()
        .buffer(&buffer)
        .editable(false)
        .monospace(true)
        .cursor_visible(false)
        .wrap_mode(WrapMode::None)
        .top_margin(12)
        .bottom_margin(12)
        .left_margin(12)
        .right_margin(12)
        .show_line_numbers(true)
        .build();

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(420)
        .child(&view)
        .build();

    SourcePane {
        widget: scroller.upcast(),
        buffer,
        view,
    }
}

fn build_text_scroller(content: &str, wrap: bool) -> gtk::Widget {
    let buffer = gtk::TextBuffer::new(None);
    buffer.set_text(content);

    let view = gtk::TextView::builder()
        .buffer(&buffer)
        .editable(false)
        .monospace(!wrap)
        .cursor_visible(false)
        .wrap_mode(if wrap {
            WrapMode::WordChar
        } else {
            WrapMode::None
        })
        .top_margin(12)
        .bottom_margin(12)
        .left_margin(12)
        .right_margin(12)
        .build();

    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(if wrap {
            gtk::PolicyType::Never
        } else {
            gtk::PolicyType::Automatic
        })
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .min_content_height(420)
        .child(&view)
        .build()
        .upcast()
}

fn build_welcome(language: Language) -> adw::StatusPage {
    let page = adw::StatusPage::builder()
        .icon_name("text-x-generic-symbolic")
        .title(tr(language, "app-title"))
        .description(tr(language, "welcome-body"))
        .vexpand(true)
        .build();
    page
}

fn show_error_dialog(window: &adw::ApplicationWindow, heading: &str, body: &str) {
    let _ = window;
    eprintln!("{heading}: {body}");
}

fn apply_source_search(buffer: &sourceview::Buffer, view: &sourceview::View, query: &str) {
    if query.is_empty() {
        return;
    }

    let start = buffer.start_iter();
    if let Some((match_start, match_end)) =
        start.forward_search(query, gtk::TextSearchFlags::CASE_INSENSITIVE, None)
    {
        buffer.select_range(&match_start, &match_end);
        view.scroll_to_iter(&mut match_start.clone(), 0.2, false, 0.0, 0.0);
    }
}

fn icon_for_kind(kind: DocumentKind) -> gio::Icon {
    let name = match kind {
        DocumentKind::Xml => "text-x-generic-symbolic",
        DocumentKind::Pdf => "application-pdf-symbolic",
    };
    gio::ThemedIcon::new(name).upcast()
}
