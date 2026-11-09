//! Android clipboard adapter.
//!
//! The desktop arboard backend does not currently compile for Android. Keep the
//! editor buildable while Android's native clipboard bridge is implemented.
//! eframe's internal clipboard simulation still supports copy/paste within the app.
use wordcraft_ui_egui::paste_picture::ClipboardPicture;

pub fn set_rich(_text: &str, _html: &str, _rtf: &str, _mathml: &str) -> bool {
    false
}

pub fn text() -> Option<String> {
    None
}

pub fn html() -> Option<String> {
    None
}

pub fn rtf() -> Option<String> {
    None
}

pub fn formats() -> Vec<String> {
    Vec::new()
}

pub fn picture() -> Option<ClipboardPicture> {
    None
}
