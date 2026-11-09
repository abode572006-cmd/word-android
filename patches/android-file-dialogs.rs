//! Android host adapter for WordCraft's file picker.
//!
//! Android's system picker is the Storage Access Framework, not an rfd desktop dialog.
//! Until the SAF Activity bridge is wired into this host, report the absence of a native
//! picker so WordCraft can use its own built-in file browser instead of compiling rfd.
use std::sync::mpsc::{channel, Receiver};

use wordcraft_ui_egui::{FileDialogAnswer, FileDialogRequest};

pub type Hook = Box<dyn Fn(FileDialogRequest) -> Receiver<FileDialogAnswer>>;

pub struct Launcher;

pub fn hook(_ctx: &egui::Context) -> (Hook, Launcher) {
    let hook: Hook = Box::new(|_request| {
        let (sender, receiver) = channel();
        let _ = sender.send(Err(
            "Android system file picker is not connected yet; using WordCraft's built-in browser".into(),
        ));
        receiver
    });
    (hook, Launcher)
}

impl Launcher {
    pub fn show(&self, _parent: &eframe::Frame) {}
}
