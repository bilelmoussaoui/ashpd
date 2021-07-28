pub fn is_empty(txt: gtk::glib::GString) -> Option<String> {
    if txt.is_empty() {
        None
    } else {
        Some(txt.to_string())
    }
}

pub fn split_comma(txt: String) -> Vec<String> {
    txt.split(',')
        .filter(|e| e.len() > 1)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
}

pub mod desktop;
mod documents;

pub use documents::DocumentsPage;
