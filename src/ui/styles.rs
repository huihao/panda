use egui::Color32;

/// Default padding for UI elements
pub const DEFAULT_PADDING: f32 = 8.0;

/// Default spacing between UI elements
pub const DEFAULT_SPACING: f32 = 4.0;

/// Application color scheme
#[derive(Clone)]
pub struct AppColors {
    /// Primary text color
    pub text: Color32,
    /// Secondary text color (for less important text)
    pub text_secondary: Color32,
    /// Accent color (for selected items, buttons, etc.)
    pub accent: Color32,
    /// Background color
    pub background: Color32,
    /// Error color
    pub error: Color32,
    /// Success color
    pub success: Color32,
    /// Warning color
    pub warning: Color32,
    /// Info color
    pub info: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            text: Color32::WHITE,
            text_secondary: Color32::from_rgb(200, 200, 200),
            accent: Color32::from_rgb(0, 150, 255),
            background: Color32::from_rgb(30, 30, 30),
            error: Color32::from_rgb(255, 50, 50),
            success: Color32::from_rgb(50, 255, 50),
            warning: Color32::from_rgb(255, 255, 50),
            info: Color32::from_rgb(50, 150, 255),
        }
    }
} 