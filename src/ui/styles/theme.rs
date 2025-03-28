use egui::{style::Margin, Color32, Rounding, Stroke, Style, Visuals};

/// Default padding for UI elements
pub const DEFAULT_PADDING: f32 = 8.0;
/// Default spacing between UI elements
pub const DEFAULT_SPACING: f32 = 4.0;
/// Default rounding radius for UI elements
pub const DEFAULT_ROUNDING: f32 = 4.0;

/// UI color scheme
#[derive(Debug, Clone)]
pub struct AppColors {
    pub background: Color32,
    pub foreground: Color32,
    pub text: Color32,
    pub text_dimmed: Color32,
    pub accent: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub success: Color32,
    pub separator: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        // Dark theme colors by default
        Self {
            background: Color32::from_rgb(32, 32, 32),
            foreground: Color32::from_rgb(45, 45, 45),
            text: Color32::from_rgb(230, 230, 230),
            text_dimmed: Color32::from_rgb(140, 140, 140),
            accent: Color32::from_rgb(66, 150, 250),
            warning: Color32::from_rgb(255, 180, 0),
            error: Color32::from_rgb(255, 85, 85),
            success: Color32::from_rgb(80, 200, 120),
            separator: Color32::from_rgb(60, 60, 60),
        }
    }
}

impl AppColors {
    /// Creates a light theme color scheme
    pub fn light() -> Self {
        Self {
            background: Color32::from_rgb(245, 245, 245),
            foreground: Color32::from_rgb(255, 255, 255),
            text: Color32::from_rgb(33, 33, 33),
            text_dimmed: Color32::from_rgb(120, 120, 120),
            accent: Color32::from_rgb(0, 120, 215),
            warning: Color32::from_rgb(255, 140, 0),
            error: Color32::from_rgb(215, 0, 0),
            success: Color32::from_rgb(0, 160, 0),
            separator: Color32::from_rgb(200, 200, 200),
        }
    }
    
    /// Creates a dark theme color scheme
    pub fn dark() -> Self {
        Self::default()
    }
}

/// Creates the default dark theme style for the application
pub fn create_dark_theme() -> Style {
    let colors = AppColors::default();
    
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(DEFAULT_SPACING, DEFAULT_SPACING);
    style.spacing.window_margin = Margin::same(DEFAULT_PADDING);
    style.spacing.button_padding = egui::vec2(DEFAULT_PADDING, DEFAULT_PADDING * 0.5);
    
    let mut visuals = Visuals::dark();
    visuals.window_rounding = Rounding::same(DEFAULT_ROUNDING);
    visuals.window_shadow.extrusion = 8.0;
    
    visuals.widgets.noninteractive.bg_fill = colors.background;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text);
    
    visuals.widgets.inactive.bg_fill = colors.foreground;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text);
    
    visuals.widgets.hovered.bg_fill = colors.accent;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    
    visuals.widgets.active.bg_fill = colors.accent;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    
    visuals.selection.bg_fill = colors.accent.linear_multiply(0.2);
    visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    
    style.visuals = visuals;
    style
}

/// Creates the default light theme style for the application
pub fn create_light_theme() -> Style {
    let colors = AppColors::light();
    
    let mut style = Style::default();
    style.spacing.item_spacing = egui::vec2(DEFAULT_SPACING, DEFAULT_SPACING);
    style.spacing.window_margin = Margin::same(DEFAULT_PADDING);
    style.spacing.button_padding = egui::vec2(DEFAULT_PADDING, DEFAULT_PADDING * 0.5);
    
    let mut visuals = Visuals::light();
    visuals.window_rounding = Rounding::same(DEFAULT_ROUNDING);
    visuals.window_shadow.extrusion = 8.0;
    
    visuals.widgets.noninteractive.bg_fill = colors.background;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors.text);
    
    visuals.widgets.inactive.bg_fill = colors.foreground;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors.text);
    
    visuals.widgets.hovered.bg_fill = colors.accent;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    
    visuals.widgets.active.bg_fill = colors.accent;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
    
    visuals.selection.bg_fill = colors.accent.linear_multiply(0.2);
    visuals.selection.stroke = Stroke::new(1.0, colors.accent);
    
    style.visuals = visuals;
    style
}