use eframe::egui::{self, Color32, RichText, Ui, Vec2, Stroke};
use eframe::epaint::CornerRadius;
use crate::gui::theme::AppTheme;

/// Create a section header with consistent styling
pub fn section_header(ui: &mut Ui, theme: &AppTheme, text: &str) {
    ui.add_space(theme.spacing_medium());
    ui.heading(RichText::new(text).color(theme.header_text).size(24.0).strong());
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(theme.spacing_small());
}

/// Create a subsection header with consistent styling
pub fn subsection_header(ui: &mut Ui, theme: &AppTheme, text: &str) {
    ui.add_space(theme.spacing_small());
    ui.label(RichText::new(text).color(theme.header_text).size(18.0).strong());
    ui.add_space(4.0);
}

/// Create a card with consistent styling
pub fn card<R>(
    ui: &mut Ui,
    theme: &AppTheme,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R
) -> R {
    ui.add_space(theme.spacing_small());

    if !title.is_empty() {
        ui.label(RichText::new(title).size(16.0).strong());
        ui.add_space(4.0);
    }

    let result = theme.card_frame().show(ui, |ui| {
        add_contents(ui)
    }).inner;

    ui.add_space(theme.spacing_small());
    result
}

/// Create a primary button with text
pub fn primary_button(ui: &mut Ui, theme: &AppTheme, text: &str) -> bool {
    theme.primary_button(ui, text)
}

/// Create a secondary button with text
pub fn secondary_button(ui: &mut Ui, theme: &AppTheme, text: &str) -> bool {
    theme.secondary_button(ui, text)
}

/// Create an accent button with text
pub fn accent_button(ui: &mut Ui, theme: &AppTheme, text: &str) -> bool {
    theme.accent_button(ui, text)
}

/// Create a sidebar button with icon and text
pub fn sidebar_button(ui: &mut Ui, theme: &AppTheme, text: &str, icon: &str, selected: bool) -> bool {
    let fill_color = if selected { theme.selected } else { theme.panel_background };
    let text_color = if selected { theme.primary } else { theme.text };

    let button = egui::Button::new(
        RichText::new(format!("{} {}", icon, text)).color(text_color).size(14.0)
    )
    .fill(fill_color)
    .min_size(Vec2::new(180.0, 36.0));

    ui.add(button).clicked()
}

/// Create a tooltip with consistent styling
pub fn tooltip(ui: &mut Ui, theme: &AppTheme, text: &str) {
    ui.label(RichText::new(text).color(theme.muted_text).italics().size(12.0));
}

/// Create a status message with consistent styling
pub fn status_message(ui: &mut Ui, theme: &AppTheme, text: &str, message_type: StatusMessageType) {
    let (bg_color, text_color) = match message_type {
        StatusMessageType::Info => (theme.panel_background, theme.text),
        StatusMessageType::Success => (theme.success, Color32::WHITE),
        StatusMessageType::Warning => (theme.warning, Color32::WHITE),
        StatusMessageType::Error => (theme.error, Color32::WHITE),
    };

    let frame = egui::Frame::new()
        .fill(bg_color)
        .stroke(Stroke::new(1.0, bg_color))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(8.0);

    frame.show(ui, |ui| {
        ui.label(RichText::new(text).color(text_color).size(14.0));
    });
}

/// Status message types
pub enum StatusMessageType {
    Info,
    Success,
    Warning,
    Error,
}

/// Create a toggle switch with label
pub fn toggle(ui: &mut Ui, theme: &AppTheme, label: &str, value: &mut bool) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.add_space(8.0);

        // Set custom colors for better visibility
        let mut style = ui.style_mut().clone();
        style.visuals.widgets.active.bg_fill = theme.primary; // Active checkbox
        style.visuals.widgets.inactive.bg_fill = theme.panel_background.gamma_multiply(1.2); // Inactive checkbox
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE); // Checkmark

        ui.style_mut().clone_from(&style);

        if ui.checkbox(value, "").changed() {
            changed = true;
        }
    });
    changed
}

/// Create a slider with label
pub fn slider<T>(ui: &mut Ui, theme: &AppTheme, label: &str, value: &mut T, range: std::ops::RangeInclusive<T>) -> bool
where
    T: egui::emath::Numeric,
{
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.add_space(8.0);

        // Create a custom slider with better visibility
        let mut slider = egui::Slider::new(value, range)
            .text("")
            .handle_shape(egui::style::HandleShape::Circle)
            .trailing_fill(true);

        // Set custom colors for better visibility
        let mut style = ui.style_mut().clone();
        style.visuals.widgets.active.bg_fill = theme.primary; // Slider active part
        style.visuals.widgets.inactive.bg_fill = theme.panel_background.gamma_multiply(1.2); // Slider background
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.primary_light); // Handle outline

        ui.style_mut().clone_from(&style);

        if ui.add(slider).changed() {
            changed = true;
        }
    });
    changed
}

/// Create a dropdown with label
pub fn dropdown<T>(
    ui: &mut Ui,
    theme: &AppTheme,
    label: &str,
    current_value: &mut T,
    options: &[T],
    format_func: impl Fn(&T) -> String
) -> bool
where
    T: PartialEq + Clone,
{
    let mut changed = false;
    let current_text = format_func(current_value);

    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.add_space(8.0);

        // Set custom colors for better visibility
        let mut style = ui.style_mut().clone();
        style.visuals.widgets.active.bg_fill = theme.primary; // Selected item
        style.visuals.widgets.hovered.bg_fill = theme.hover; // Hovered item
        style.visuals.widgets.inactive.bg_fill = theme.panel_background.gamma_multiply(1.2); // Dropdown background

        ui.style_mut().clone_from(&style);

        egui::ComboBox::from_label("")
            .selected_text(RichText::new(current_text).color(theme.text))
            .show_ui(ui, |ui| {
                for option in options {
                    let option_text = format_func(option);
                    if ui.selectable_label(option == current_value, option_text).clicked() {
                        *current_value = option.clone();
                        changed = true;
                    }
                }
            });
    });

    changed
}

/// Create an animated progress bar
pub fn progress_bar(ui: &mut Ui, theme: &AppTheme, progress: f32, text: Option<&str>) {
    // Create a frame around the progress bar for better visibility
    let frame = egui::Frame::new()
        .fill(theme.panel_background.gamma_multiply(1.2))
        .stroke(Stroke::new(1.0, theme.border))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(4.0);

    frame.show(ui, |ui| {
        let progress_bar = egui::ProgressBar::new(progress)
            .animate(true)
            .fill(theme.primary)
            .show_percentage();

        if let Some(text) = text {
            ui.add(progress_bar.text(RichText::new(text).strong()));
        } else {
            ui.add(progress_bar);
        }
    });
}
