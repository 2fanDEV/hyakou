use egui::{Color32, FontFamily};

use crate::gui::{
    BOLD_FONT_FAMILY,
    widgets::text_editor::{
        MAX_FONT_SIZE, MIN_FONT_SIZE, TextEditor, TextEditorResponse, TextEditorSettings,
    },
};

#[test]
fn test_text_editor_stores_text() {
    let mut editor = TextEditor::new("editor", "hello");

    assert_eq!(editor.text(), "hello");

    editor.set_text("updated");

    assert_eq!(editor.text(), "updated");
}

#[test]
fn test_text_editor_settings_default_to_editable_text_values() {
    let settings = TextEditorSettings::default();

    assert_eq!(settings.font_size, 16.0);
    assert!(!settings.bold);
    assert_eq!(settings.color, Color32::WHITE);
}

#[test]
fn test_text_editor_defaults_to_singleline_editable_mode() {
    let editor = TextEditor::new("editor", "hello");

    assert!(!editor.multiline());
    assert!(!editor.read_only());
}

#[test]
fn test_text_editor_settings_store_font_size_boldness_and_color() {
    let mut editor = TextEditor::new("editor", "hello");
    *editor.settings_mut() = TextEditorSettings {
        font_size: 24.0,
        bold: true,
        color: Color32::from_rgba_unmultiplied(20, 40, 60, 128),
    };

    assert_eq!(
        editor.settings(),
        TextEditorSettings {
            font_size: 24.0,
            bold: true,
            color: Color32::from_rgba_unmultiplied(20, 40, 60, 128),
        }
    );
}

#[test]
fn test_bold_setting_changes_text_format_family() {
    let normal_format = TextEditor::text_format(TextEditorSettings {
        font_size: 16.0,
        bold: false,
        color: Color32::WHITE,
    });
    let bold_format = TextEditor::text_format(TextEditorSettings {
        font_size: 16.0,
        bold: true,
        color: Color32::WHITE,
    });

    assert_eq!(normal_format.font_id.family, FontFamily::Proportional);
    assert_eq!(
        bold_format.font_id.family,
        FontFamily::Name(BOLD_FONT_FAMILY.into())
    );
}

#[test]
fn test_text_editor_can_be_multiline() {
    let mut editor = TextEditor::new("editor", "hello");

    editor.set_multiline(true);

    assert!(editor.multiline());
}

#[test]
fn test_text_layout_respects_multiline_mode() {
    let mut singleline_editor = TextEditor::new("singleline", "hello\nworld");
    let mut multiline_editor = TextEditor::new("multiline", "hello\nworld");
    multiline_editor.set_multiline(true);

    egui::__run_test_ui(|ui| {
        singleline_editor.show(ui);
        multiline_editor.show(ui);
    });

    assert!(!singleline_editor.multiline());
    assert!(multiline_editor.multiline());
}

#[test]
fn test_text_editor_clamps_font_size_during_show() {
    let mut small_editor = TextEditor::new("small", "hello");
    small_editor.settings_mut().font_size = 1.0;

    egui::__run_test_ui(|ui| {
        small_editor.show(ui);
    });

    assert_eq!(small_editor.settings().font_size, MIN_FONT_SIZE);

    let mut large_editor = TextEditor::new("large", "hello");
    large_editor.settings_mut().font_size = 500.0;

    egui::__run_test_ui(|ui| {
        large_editor.show(ui);
    });

    assert_eq!(large_editor.settings().font_size, MAX_FONT_SIZE);
}

#[test]
fn test_text_editor_response_records_focus_state() {
    let response = TextEditorResponse {
        changed: true,
        has_focus: true,
        gained_focus: false,
        lost_focus: false,
    };

    assert!(response.changed);
    assert!(response.has_focus);
    assert!(!response.gained_focus);
    assert!(!response.lost_focus);
}

#[test]
fn test_toolbar_starts_closed() {
    let editor = TextEditor::new("editor", "hello");

    assert!(!editor.toolbar_open());
    assert!(editor.toolbar_enabled());
}

#[test]
fn test_toolbar_can_be_disabled() {
    let mut editor = TextEditor::new("editor", "hello");

    editor.set_toolbar_enabled(false);

    assert!(!editor.toolbar_enabled());
    assert!(!editor.toolbar_open());
}

#[test]
fn test_read_only_mode_closes_toolbar() {
    let mut editor = TextEditor::new("editor", "hello");

    editor.set_read_only(true);

    assert!(editor.read_only());
    assert!(!editor.toolbar_open());
}

#[test]
fn test_read_only_mode_stays_display_only_when_shown() {
    let mut editor = TextEditor::new("editor", "hello");
    editor.set_read_only(true);

    egui::__run_test_ui(|ui| {
        let response = editor.show(ui);

        assert!(!response.changed);
        assert!(!response.has_focus);
    });

    assert!(!editor.toolbar_open());
}

#[test]
fn test_toolbar_stays_open_without_outside_click() {
    assert!(!TextEditor::should_close_toolbar(false, None));
}

#[test]
fn test_toolbar_closes_when_text_is_clicked_elsewhere_without_toolbar_response() {
    assert!(TextEditor::should_close_toolbar(true, None));
}
