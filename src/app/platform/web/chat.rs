//! Web Chat
#![allow(dead_code)]

use egui::Ui;

pub struct ChatWidget;

impl ChatWidget {
    fn ui(&mut self, ui: &mut Ui) {
        ui.label("Chat");
    }
}
