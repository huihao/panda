use egui::Color32;

#[derive(Clone)]
pub struct AppColors {
    pub text: Color32,
    pub text_dim: Color32,
    pub text_highlight: Color32,
    pub background: Color32,
    pub background_highlight: Color32,
    pub accent: Color32,
    pub error: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            text: Color32::from_rgb(220, 220, 220),
            text_dim: Color32::from_rgb(140, 140, 140),
            text_highlight: Color32::from_rgb(255, 255, 255),
            background: Color32::from_rgb(30, 30, 30),
            background_highlight: Color32::from_rgb(45, 45, 45),
            accent: Color32::from_rgb(0, 120, 215),
            error: Color32::from_rgb(255, 85, 85),
        }
    }
}

pub const DEFAULT_PADDING: f32 = 8.0;
pub const DEFAULT_SPACING: f32 = 4.0;