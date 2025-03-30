use egui::Color32;

#[derive(Clone, Copy)]
pub struct AppColors {
    pub background: Color32,
    pub text: Color32,
    pub text_secondary: Color32,
    pub text_dimmed: Color32,
    pub accent: Color32,
    pub error: Color32,
    pub success: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            background: Color32::from_rgb(30, 30, 30),
            text: Color32::WHITE,
            text_secondary: Color32::from_rgb(200, 200, 200),
            text_dimmed: Color32::from_rgb(150, 150, 150),
            accent: Color32::from_rgb(0, 150, 255),
            error: Color32::from_rgb(255, 50, 50),
            success: Color32::from_rgb(50, 255, 50),
        }
    }
} 