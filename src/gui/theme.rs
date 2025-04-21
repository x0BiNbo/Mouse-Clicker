use eframe::egui::{self, Color32, Stroke, Vec2, Ui, RichText};
use eframe::epaint::{CornerRadius, Margin};

/// Modern theme with smooth colors and consistent styling
#[derive(Clone)]
pub struct AppTheme {
    // Primary colors
    pub primary: Color32,
    pub primary_light: Color32,
    pub primary_dark: Color32,

    // Secondary colors
    pub secondary: Color32,
    pub accent: Color32,

    // Background colors
    pub background: Color32,
    pub card_background: Color32,
    pub panel_background: Color32,

    // Text colors
    pub text: Color32,
    pub muted_text: Color32,
    pub header_text: Color32,

    // Interactive colors
    pub hover: Color32,
    pub active: Color32,
    pub selected: Color32,

    // Status colors
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,

    // Border colors
    pub border: Color32,

    // Animation settings
    pub animation_duration: f32,
}

impl AppTheme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            // Primary colors - light green accent
            primary: Color32::from_rgb(100, 210, 120),     // Light green
            primary_light: Color32::from_rgb(140, 230, 150),
            primary_dark: Color32::from_rgb(80, 180, 100),

            // Secondary colors
            secondary: Color32::from_rgb(255, 165, 80),    // Light orange
            accent: Color32::from_rgb(255, 100, 80),       // Coral

            // Background colors - dark with slight blue tint
            background: Color32::from_rgb(22, 22, 28),
            card_background: Color32::from_rgb(32, 32, 40),
            panel_background: Color32::from_rgb(28, 28, 36),

            // Text colors
            text: Color32::from_rgb(240, 240, 240),
            muted_text: Color32::from_rgb(180, 180, 190),
            header_text: Color32::from_rgb(255, 255, 255),

            // Interactive colors
            hover: Color32::from_rgb(45, 45, 55),
            active: Color32::from_rgb(55, 55, 65),
            selected: Color32::from_rgb(50, 60, 50),       // Slight green tint for selected

            // Status colors
            success: Color32::from_rgb(100, 210, 120),     // Light green
            warning: Color32::from_rgb(255, 165, 80),      // Light orange
            error: Color32::from_rgb(255, 100, 80),        // Coral

            // Border colors
            border: Color32::from_rgb(50, 50, 60),

            // Animation settings
            animation_duration: 0.15,
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        Self {
            // Primary colors - deep blue with purple tint
            primary: Color32::from_rgb(63, 81, 181),       // Indigo
            primary_light: Color32::from_rgb(92, 107, 192),
            primary_dark: Color32::from_rgb(48, 63, 159),

            // Secondary colors
            secondary: Color32::from_rgb(33, 150, 243),    // Blue
            accent: Color32::from_rgb(233, 30, 99),        // Pink

            // Background colors
            background: Color32::from_rgb(245, 245, 250),
            card_background: Color32::from_rgb(255, 255, 255),
            panel_background: Color32::from_rgb(250, 250, 255),

            // Text colors
            text: Color32::from_rgb(33, 33, 33),
            muted_text: Color32::from_rgb(120, 120, 120),
            header_text: Color32::from_rgb(10, 10, 10),

            // Interactive colors
            hover: Color32::from_rgb(240, 240, 245),
            active: Color32::from_rgb(230, 230, 240),
            selected: Color32::from_rgb(235, 235, 255),

            // Status colors
            success: Color32::from_rgb(76, 175, 80),       // Green
            warning: Color32::from_rgb(255, 152, 0),       // Orange
            error: Color32::from_rgb(244, 67, 54),         // Red

            // Border colors
            border: Color32::from_rgb(220, 220, 230),

            // Animation settings
            animation_duration: 0.15,
        }
    }

    /// Apply the theme to the context
    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Update visuals
        style.visuals.window_fill = self.background;
        style.visuals.panel_fill = self.panel_background;
        style.visuals.faint_bg_color = self.card_background;
        style.visuals.extreme_bg_color = self.background;

        // Text colors
        style.visuals.override_text_color = Some(self.text);

        // Widget colors
        style.visuals.widgets.noninteractive.bg_fill = self.panel_background;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text);

        style.visuals.widgets.inactive.bg_fill = self.panel_background;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text);

        style.visuals.widgets.hovered.bg_fill = self.hover;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, self.text);

        style.visuals.widgets.active.bg_fill = self.active;
        style.visuals.widgets.active.fg_stroke = Stroke::new(2.0, self.primary);

        style.visuals.widgets.open.bg_fill = self.selected;
        style.visuals.widgets.open.fg_stroke = Stroke::new(2.0, self.primary);

        // Selection color
        style.visuals.selection.bg_fill = self.selected;
        style.visuals.selection.stroke = Stroke::new(1.0, self.primary);

        // Window rounding
        style.visuals.window_corner_radius = 8.0.into();
        style.visuals.menu_corner_radius = 6.0.into();

        // Apply the style
        ctx.set_style(style);
    }

    /// Create a primary button
    pub fn primary_button(&self, ui: &mut Ui, text: &str) -> bool {
        let button = egui::Button::new(
            RichText::new(text).color(Color32::WHITE).size(14.0)
        )
        .fill(self.primary)
        .stroke(Stroke::new(1.0, self.primary_dark))
        .corner_radius(CornerRadius::same(4));

        ui.add(button).clicked()
    }

    /// Create a secondary button
    pub fn secondary_button(&self, ui: &mut Ui, text: &str) -> bool {
        let button = egui::Button::new(
            RichText::new(text).color(Color32::WHITE).size(14.0)
        )
        .fill(self.secondary)
        .stroke(Stroke::new(1.0, self.secondary))
        .corner_radius(CornerRadius::same(4));

        ui.add(button).clicked()
    }

    /// Create an accent button
    pub fn accent_button(&self, ui: &mut Ui, text: &str) -> bool {
        let button = egui::Button::new(
            RichText::new(text).color(Color32::WHITE).size(14.0)
        )
        .fill(self.accent)
        .stroke(Stroke::new(1.0, self.accent))
        .corner_radius(CornerRadius::same(4));

        ui.add(button).clicked()
    }

    /// Create a card frame
    pub fn card_frame(&self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.card_background)
            .stroke(Stroke::new(1.0, self.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(16.0)
            .outer_margin(8.0)
    }

    /// Create a panel frame
    pub fn panel_frame(&self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.panel_background)
            .stroke(Stroke::new(1.0, self.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(16.0)
            .outer_margin(8.0)
    }

    /// Create a section frame
    pub fn section_frame(&self) -> egui::Frame {
        egui::Frame::new()
            .fill(self.panel_background)
            .stroke(Stroke::new(1.0, self.border))
            .corner_radius(CornerRadius::same(4))
            .inner_margin(12.0)
            .outer_margin(4.0)
    }

    /// Standard spacing between elements
    pub fn spacing_small(&self) -> f32 {
        8.0
    }

    /// Medium spacing between sections
    pub fn spacing_medium(&self) -> f32 {
        16.0
    }

    /// Large spacing for major sections
    pub fn spacing_large(&self) -> f32 {
        24.0
    }

    /// Standard icon size
    pub fn icon_size(&self) -> Vec2 {
        Vec2::new(20.0, 20.0)
    }
}
