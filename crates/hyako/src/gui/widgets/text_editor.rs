use std::{hash::Hash, sync::Arc};

use egui::text::{Galley, LayoutJob, TextFormat};
use egui::{
    Color32, FontFamily, FontId, Id, Order, Pos2, Response, RichText, TextBuffer, TextEdit, Ui,
};

use crate::gui::BOLD_FONT_FAMILY;

const DEFAULT_FONT_SIZE: f32 = 16.0;
const MIN_FONT_SIZE: f32 = 8.0;
const MAX_FONT_SIZE: f32 = 96.0;
const TOOLBAR_GAP: f32 = 8.0;
const DEFAULT_ROWS: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextEditorSettings {
    pub font_size: f32,
    pub bold: bool,
    pub color: Color32,
}

impl Default for TextEditorSettings {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            bold: false,
            color: Color32::WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextEditorResponse {
    pub changed: bool,
    pub has_focus: bool,
    pub gained_focus: bool,
    pub lost_focus: bool,
}

#[derive(Debug, Clone)]
pub struct TextEditor {
    id: Id,
    text: String,
    settings: TextEditorSettings,
    toolbar_enabled: bool,
    toolbar_open: bool,
    multiline: bool,
    read_only: bool,
}

impl TextEditor {
    pub fn new(id_salt: impl Hash, text: impl Into<String>) -> Self {
        Self {
            id: Id::new(id_salt),
            text: text.into(),
            settings: TextEditorSettings::default(),
            toolbar_enabled: true,
            toolbar_open: false,
            multiline: false,
            read_only: false,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    pub fn settings(&self) -> TextEditorSettings {
        self.settings
    }

    pub fn settings_mut(&mut self) -> &mut TextEditorSettings {
        &mut self.settings
    }

    pub fn toolbar_open(&self) -> bool {
        self.toolbar_open
    }

    pub fn toolbar_enabled(&self) -> bool {
        self.toolbar_enabled
    }

    pub fn set_toolbar_enabled(&mut self, enabled: bool) {
        self.toolbar_enabled = enabled;
        if !enabled {
            self.toolbar_open = false;
        }
    }

    pub fn multiline(&self) -> bool {
        self.multiline
    }

    pub fn set_multiline(&mut self, multiline: bool) {
        self.multiline = multiline;
    }

    pub fn read_only(&self) -> bool {
        self.read_only
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
        if read_only {
            self.toolbar_open = false;
        }
    }

    pub fn show(&mut self, ui: &mut Ui) -> TextEditorResponse {
        self.clamp_settings();
        if self.read_only {
            return self.show_read_only(ui);
        }

        let settings = self.settings;
        let multiline = self.multiline;
        let mut layouter = |ui: &Ui, text: &dyn TextBuffer, wrap_width: f32| {
            Self::layout_text(ui, text, wrap_width, settings, multiline)
        };

        let response = ui.add(self.text_edit().layouter(&mut layouter));

        let text_response = TextEditorResponse::from_response(&response);
        if self.toolbar_enabled && (text_response.has_focus || text_response.gained_focus) {
            self.toolbar_open = true;
        }

        let toolbar_response = if self.toolbar_enabled && self.toolbar_open {
            Some(self.show_toolbar(ui, response.rect.left_top()))
        } else {
            None
        };

        if Self::should_close_toolbar(response.clicked_elsewhere(), toolbar_response.as_ref()) {
            self.toolbar_open = false;
        }

        text_response
    }

    fn show_read_only(&mut self, ui: &mut Ui) -> TextEditorResponse {
        self.toolbar_open = false;
        let response = ui.label(
            RichText::new(self.text.as_str())
                .font(Self::font_id(self.settings))
                .color(self.settings.color),
        );
        TextEditorResponse::from_response(&response)
    }

    fn text_edit(&mut self) -> TextEdit<'_> {
        let text_edit = if self.multiline {
            TextEdit::multiline(&mut self.text).desired_rows(DEFAULT_ROWS)
        } else {
            TextEdit::singleline(&mut self.text).desired_rows(DEFAULT_ROWS)
        };

        text_edit
            .id(self.id)
            .font(Self::font_id(self.settings))
            .text_color(self.settings.color)
    }

    fn show_toolbar(&mut self, ui: &mut Ui, text_field_pos: Pos2) -> Response {
        let toolbar_pos = Pos2::new(text_field_pos.x, (text_field_pos.y - TOOLBAR_GAP).max(0.0));
        let inner = egui::Area::new(self.id.with("toolbar"))
            .order(Order::Foreground)
            .fixed_pos(toolbar_pos)
            .pivot(egui::Align2::LEFT_BOTTOM)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Size");
                        ui.add(
                            egui::DragValue::new(&mut self.settings.font_size)
                                .range(MIN_FONT_SIZE..=MAX_FONT_SIZE)
                                .speed(0.25),
                        );
                        ui.checkbox(&mut self.settings.bold, "Bold");
                        ui.color_edit_button_srgba(&mut self.settings.color);
                    });
                });
            });

        self.clamp_settings();
        inner.response
    }

    fn clamp_settings(&mut self) {
        self.settings.font_size = self.settings.font_size.clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
    }

    fn layout_text(
        ui: &Ui,
        text: &dyn TextBuffer,
        wrap_width: f32,
        settings: TextEditorSettings,
        multiline: bool,
    ) -> Arc<Galley> {
        let mut job =
            LayoutJob::single_section(text.as_str().to_owned(), Self::text_format(settings));
        job.wrap.max_width = wrap_width;
        job.break_on_newline = multiline;
        ui.fonts_mut(|fonts| fonts.layout_job(job))
    }

    fn text_format(settings: TextEditorSettings) -> TextFormat {
        TextFormat::simple(Self::font_id(settings), settings.color)
    }

    fn font_id(settings: TextEditorSettings) -> FontId {
        if settings.bold {
            FontId::new(
                settings.font_size,
                FontFamily::Name(BOLD_FONT_FAMILY.into()),
            )
        } else {
            FontId::proportional(settings.font_size)
        }
    }

    fn should_close_toolbar(
        clicked_away_from_text: bool,
        toolbar_response: Option<&Response>,
    ) -> bool {
        clicked_away_from_text && toolbar_response.is_none_or(Response::clicked_elsewhere)
    }
}

impl TextEditorResponse {
    fn from_response(response: &Response) -> Self {
        Self {
            changed: response.changed(),
            has_focus: response.has_focus(),
            gained_focus: response.gained_focus(),
            lost_focus: response.lost_focus(),
        }
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new("text_editor", String::new())
    }
}

#[cfg(test)]
#[path = "text_editor_tests.rs"]
mod tests;
