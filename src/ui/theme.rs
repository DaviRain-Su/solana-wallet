use gpui::*;

#[derive(Clone)]
pub struct Theme {
    // Background colors
    pub sidebar_bg: Hsla,
    pub content_bg: Hsla,
    pub card_bg: Hsla,
    
    // Text colors
    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_muted: Hsla,
    
    // Navigation colors
    pub nav_active_bg: Hsla,
    pub nav_active_text: Hsla,
    pub nav_hover_bg: Hsla,
    
    // Border colors
    pub border: Hsla,
    pub border_focused: Hsla,
    
    // Status colors
    pub success: Hsla,
    pub warning: Hsla,
    pub error: Hsla,
    pub info: Hsla,
    
    // Button colors
    pub button_primary_bg: Hsla,
    pub button_primary_hover: Hsla,
    pub button_primary_text: Hsla,
    pub button_secondary_bg: Hsla,
    pub button_secondary_hover: Hsla,
    pub button_secondary_text: Hsla,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            // Background colors
            sidebar_bg: hsla(220.0, 0.13, 0.18, 1.0),
            content_bg: hsla(220.0, 0.13, 0.10, 1.0),
            card_bg: hsla(220.0, 0.13, 0.15, 1.0),
            
            // Text colors
            text_primary: hsla(0.0, 0.0, 0.95, 1.0),
            text_secondary: hsla(0.0, 0.0, 0.7, 1.0),
            text_muted: hsla(0.0, 0.0, 0.5, 1.0),
            
            // Navigation colors
            nav_active_bg: hsla(266.0, 0.8, 0.5, 0.2),
            nav_active_text: hsla(266.0, 0.8, 0.7, 1.0),
            nav_hover_bg: hsla(0.0, 0.0, 1.0, 0.05),
            
            // Border colors
            border: hsla(0.0, 0.0, 1.0, 0.1),
            border_focused: hsla(266.0, 0.8, 0.5, 0.5),
            
            // Status colors
            success: hsla(142.0, 0.71, 0.45, 1.0),
            warning: hsla(38.0, 0.92, 0.5, 1.0),
            error: hsla(0.0, 0.84, 0.6, 1.0),
            info: hsla(201.0, 0.96, 0.54, 1.0),
            
            // Button colors
            button_primary_bg: hsla(266.0, 0.8, 0.5, 1.0),
            button_primary_hover: hsla(266.0, 0.8, 0.6, 1.0),
            button_primary_text: hsla(0.0, 0.0, 1.0, 1.0),
            button_secondary_bg: hsla(0.0, 0.0, 1.0, 0.1),
            button_secondary_hover: hsla(0.0, 0.0, 1.0, 0.15),
            button_secondary_text: hsla(0.0, 0.0, 0.9, 1.0),
        }
    }
    
    pub fn light() -> Self {
        Self {
            // Background colors
            sidebar_bg: hsla(0.0, 0.0, 0.98, 1.0),
            content_bg: hsla(0.0, 0.0, 1.0, 1.0),
            card_bg: hsla(0.0, 0.0, 0.98, 1.0),
            
            // Text colors
            text_primary: hsla(0.0, 0.0, 0.1, 1.0),
            text_secondary: hsla(0.0, 0.0, 0.3, 1.0),
            text_muted: hsla(0.0, 0.0, 0.5, 1.0),
            
            // Navigation colors
            nav_active_bg: hsla(266.0, 0.8, 0.5, 0.1),
            nav_active_text: hsla(266.0, 0.8, 0.5, 1.0),
            nav_hover_bg: hsla(0.0, 0.0, 0.0, 0.05),
            
            // Border colors
            border: hsla(0.0, 0.0, 0.0, 0.1),
            border_focused: hsla(266.0, 0.8, 0.5, 0.5),
            
            // Status colors
            success: hsla(142.0, 0.71, 0.45, 1.0),
            warning: hsla(38.0, 0.92, 0.5, 1.0),
            error: hsla(0.0, 0.84, 0.6, 1.0),
            info: hsla(201.0, 0.96, 0.54, 1.0),
            
            // Button colors
            button_primary_bg: hsla(266.0, 0.8, 0.5, 1.0),
            button_primary_hover: hsla(266.0, 0.8, 0.4, 1.0),
            button_primary_text: hsla(0.0, 0.0, 1.0, 1.0),
            button_secondary_bg: hsla(0.0, 0.0, 0.0, 0.05),
            button_secondary_hover: hsla(0.0, 0.0, 0.0, 0.1),
            button_secondary_text: hsla(0.0, 0.0, 0.1, 1.0),
        }
    }
}

pub fn get_theme(cx: &WindowContext) -> Theme {
    // TODO: Read from user preferences
    Theme::dark()
}