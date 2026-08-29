use eframe::egui;
use evdev::KeyCode;

const RISE_TIME: f32 = 0.01;
const FALL_TIME: f32 = 0.2;

const WATERFALL_SPEED: f32 = 300.0;
pub(crate) const WATERFALL_HEIGHT: f32 = 500.0;
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
        let just_pressed = self.is_down && !self.was_down;
        let just_released = !self.is_down && self.was_down;

        if self.is_down {
            self.t = (self.t + dt / RISE_TIME).min(1.0);
        } else {
            self.t = (self.t - dt / FALL_TIME).max(0.0);
        }

        self.was_down = self.is_down;

        let eased_t = ease_out(self.t);

        // growing
        if let Some(last) = self.segments.last_mut() {
            if last.growing {
                last.height += WATERFALL_SPEED * dt;
                last.y_offset -= WATERFALL_SPEED * dt; // dirty hack who cares
            }
        }

        // offset waterfall
        for segment in self.segments.iter_mut() {
            segment.y_offset += WATERFALL_SPEED * dt;
        }

        self.segments.retain(|s| s.y_offset < WATERFALL_HEIGHT); // cleanup

        // waterfall visuals
        if just_pressed {
            self.segments.push(Segment {
                height: 0.0,
                y_offset: 0.0,
                growing: true,
            });
            self.press_count += 1;
        }
        if just_released {
            if let Some(seg) = self.segments.iter_mut().find(|s| s.growing) {
                seg.growing = false;
            }
        }

        eased_t
    }
}

pub struct Segment {
    pub height: f32,
    pub y_offset: f32,
    pub growing: bool,
}
