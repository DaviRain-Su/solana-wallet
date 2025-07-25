use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub struct Theme {
    pub mode: ThemeMode,
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_hover: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_disabled: Rgba,
    pub border: Rgba,
    pub primary: Rgba,
    pub primary_hover: Rgba,
    pub success: Rgba,
    pub error: Rgba,
    pub warning: Rgba,
}

impl Theme {
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            background: rgb(0xffffff),
            surface: rgb(0xfafafa),
            surface_hover: rgb(0xf0f0f0),
            text_primary: rgb(0x1a1a1a),
            text_secondary: rgb(0x4a4a4a),
            text_disabled: rgb(0x9a9a9a),
            border: rgb(0xd0d0d0),
            primary: rgb(0x9945ff),
            primary_hover: rgb(0x7733dd),
            success: rgb(0x00a547),
            error: rgb(0xe53935),
            warning: rgb(0xff9800),
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            background: rgb(0x1e1e1e),
            surface: rgb(0x2a2a2a),
            surface_hover: rgb(0x3a3a3a),
            text_primary: rgb(0xffffff),
            text_secondary: rgb(0xaaaaaa),
            text_disabled: rgb(0x666666),
            border: rgb(0x404040),
            primary: rgb(0x9945ff),
            primary_hover: rgb(0xaa56ff),
            success: rgb(0x00ff00),
            error: rgb(0xff6666),
            warning: rgb(0xffaa00),
        }
    }

    pub fn card_background(&self) -> Rgba {
        self.surface
    }

    pub fn button_primary(&self) -> Rgba {
        self.primary
    }

    pub fn button_primary_hover(&self) -> Rgba {
        self.primary_hover
    }

    pub fn button_ghost(&self) -> Rgba {
        if self.mode == ThemeMode::Light {
            rgb(0xf0f0f0)
        } else {
            rgb(0x333333)
        }
    }

    pub fn button_ghost_hover(&self) -> Rgba {
        if self.mode == ThemeMode::Light {
            rgb(0xe0e0e0)
        } else {
            rgb(0x444444)
        }
    }
}

// 全局主题状态
pub struct ThemeState {
    pub current: Theme,
}

impl ThemeState {
    pub fn new() -> Self {
        Self {
            current: Theme::dark(),
        }
    }

    pub fn toggle(&mut self) {
        self.current = match self.current.mode {
            ThemeMode::Light => Theme::dark(),
            ThemeMode::Dark => Theme::light(),
        };
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.current = match mode {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
        };
    }
}
