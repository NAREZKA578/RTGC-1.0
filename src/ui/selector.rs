use crate::graphics::ui_renderer::batch::{Color, DrawBatch};

pub struct Selector {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub items: Vec<String>,
    pub selected_index: usize,
}

impl Selector {
    pub fn new(x: f32, y: f32, width: f32, items: Vec<String>) -> Self {
        Self {
            x,
            y,
            width,
            items,
            selected_index: 0,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    pub fn prev(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    pub fn current(&self) -> &str {
        if self.items.is_empty() {
            return "";
        }
        &self.items[self.selected_index]
    }
}
