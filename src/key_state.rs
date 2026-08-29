use eframe::egui;
use evdev::KeyCode;

pub const KEYS: [(KeyCode, &str); 4] = [
    (KeyCode::KEY_S, "S"),
    (KeyCode::KEY_D, "D"),
    (KeyCode::KEY_K, "K"),
    (KeyCode::KEY_L, "L"),
];

pub struct KeyVisualState {
    pub color: egui::Color32,
    pub scale: egui::Vec2,
    pub offset: egui::Pos2,
}

pub const KEY_UP: KeyVisualState = KeyVisualState {
    color: egui::Color32::from_gray(60),
    scale: egui::Vec2::new(1.0, 1.0),
    offset: egui::Pos2::new(0.0, 0.0),
};
pub const KEY_DOWN: KeyVisualState = KeyVisualState {
    color: egui::Color32::from_gray(150),
    scale: egui::Vec2::new(1.05, 1.05),
    offset: egui::Pos2::new(0.0, 2.0),
};

pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

pub struct KeyState {
    pub keycode: KeyCode,
    pub label: String,
    pub is_down: bool,
    pub was_down: bool,
    pub press_count: u32,
    pub t: f32,
    pub segments: Vec<Segment>,
}

impl KeyState {
    pub fn new(keycode: KeyCode, label: String) -> Self {
        KeyState {
            keycode,
            label,
            is_down: false,
            was_down: false,
            press_count: 0,
            t: 0.0,
            segments: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32) -> f32 {
        
    }
}

pub struct Segment {
    pub height: f32,
    pub y_offset: f32,
    pub growing: bool,
}





