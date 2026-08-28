use eframe::egui::{self};
use evdev::{EventSummary, KeyCode};
use std::sync::{Arc, Mutex};

const KEYS: [(KeyCode, &str); 4] = [
    (KeyCode::KEY_S, "S"),
    (KeyCode::KEY_D, "D"),
    (KeyCode::KEY_K, "K"),
    (KeyCode::KEY_L, "L"),
];

struct KeyVisualState {
    color: egui::Color32,
    scale: egui::Vec2,
    offset: egui::Pos2,
}
struct KpsApp {
    keys: Arc<Mutex<Vec<KeyState>>>,
    last_frame: std::time::Instant,
}

const BUTTON_WIDTH: f32 = 50.0;
const BUTTON_HEIGHT: f32 = 50.0;
const GAP: f32 = 10.0;
const RISE_TIME: f32 = 0.01;
const FALL_TIME: f32 = 0.2;

const KEY_UP: KeyVisualState = KeyVisualState {
    color: egui::Color32::from_gray(60),
    scale: egui::Vec2::new(1.0, 1.0),
    offset: egui::Pos2::new(0.0, 0.0),
};
const KEY_DOWN: KeyVisualState = KeyVisualState {
    color: egui::Color32::from_gray(150),
    scale: egui::Vec2::new(1.05, 1.05),
    offset: egui::Pos2::new(0.0, 2.0),
};

// easing
fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(2)
}

impl KpsApp {
    fn new() -> Self {
        let mut keys = Vec::new();
        for (keycode, label) in KEYS {
            keys.push(KeyState::new(keycode, label.to_string()));
        }
        let keys = Arc::new(Mutex::new(keys));

        let keys_clone = Arc::clone(&keys);
        std::thread::spawn(move || {
            let mut device = evdev::Device::open("/dev/input/event8").unwrap();
            loop {
                for event in device.fetch_events().unwrap() {
                    match event.destructure() {
                        EventSummary::Key(_, keycode, value) => {
                            let mut keys = keys_clone.lock().unwrap();
                            if let Some(key) = keys.iter_mut().find(|k| k.keycode == keycode) {
                                key.is_down = value != 0;
                            }
                        }
                        _ => {}
                    }
                }
            }
        });

        KpsApp {
            keys,
            last_frame: std::time::Instant::now(),
        }
    }
}

impl eframe::App for KpsApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.ctx().request_repaint(); // redraw anyway
        let now = std::time::Instant::now();
        let dt = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;

        egui::CentralPanel::default().show(ui, |ui| {
            let painter = ui.painter();
            for (i, key) in self.keys.lock().unwrap().iter_mut().enumerate() {
                if key.is_down {
                    key.t = (key.t + dt / RISE_TIME).min(1.0);
                } else {
                    key.t = (key.t - dt / FALL_TIME).max(0.0);
                }
                let eased_t = ease_out(key.t);

                let x = GAP + i as f32 * (BUTTON_WIDTH + GAP);
                let y: f32 = 10.0;
                let base_center = egui::Pos2::new(
                    x + BUTTON_WIDTH / 2.0,
                    y + BUTTON_HEIGHT / 2.0,
                );
                let offset = KEY_UP.offset.lerp(KEY_DOWN.offset, eased_t);
                let center = base_center + offset.to_vec2();
                let size = egui::vec2(BUTTON_WIDTH, BUTTON_HEIGHT)
                    * (KEY_UP.scale + (KEY_DOWN.scale - KEY_UP.scale) * eased_t);
                let rect = egui::Rect::from_center_size(center, size);

                // box
                let color = KEY_UP.color.lerp_to_gamma(KEY_DOWN.color, eased_t);

                painter.rect_filled(rect, 0.0, color);

                // text
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    key.label.to_string(),
                    egui::FontId::default(),
                    egui::Color32::WHITE,
                );
            }
        });
    }
}

struct KeyState {
    keycode: KeyCode,
    label: String,
    is_down: bool,
    was_down: bool,
    press_count: u32,
    t: f32,
}

impl KeyState {
    fn new(keycode: KeyCode, label: String) -> Self {
        KeyState {
            keycode,
            label,
            is_down: false,
            was_down: false,
            press_count: 0,
            t: 0.0,
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "keypress",
        options,
        Box::new(|_cc| Ok(Box::new(KpsApp::new()))),
    )
}
