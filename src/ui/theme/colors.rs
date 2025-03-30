use egui::Color32;

/// Application color theme
#[derive(Debug, Clone)]
pub struct AppColors {
    /// Primary text color
    pub text_primary: Color32,
    
    /// Secondary text color
    pub text_secondary: Color32,
    
    /// Dimmed text color
    pub text_dimmed: Color32,
    
    /// Regular text color
    pub text: Color32,
    
    /// Accent color
    pub accent: Color32,
    
    /// Background color
    pub background: Color32,
    
    /// Error color
    pub error: Color32,
    
    /// Success color
    pub success: Color32,
}

impl Default for AppColors {
    fn default() -> Self {
        Self {
            text_primary: Color32::from_rgb(255, 255, 255),
            text_secondary: Color32::from_rgb(200, 200, 200),
            text_dimmed: Color32::from_rgb(150, 150, 150),
            text: Color32::from_rgb(255, 255, 255),
            accent: Color32::from_rgb(0, 150, 255),
            background: Color32::from_rgb(30, 30, 30),
            error: Color32::from_rgb(255, 50, 50),
            success: Color32::from_rgb(50, 255, 50),
        }
    }
} 