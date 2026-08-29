mod key_state;

use eframe::egui::{self};
use evdev::EventSummary;
use std::sync::{Arc, Mutex};

use key_state::{KEY_DOWN, KEY_UP, KEYS, KeyState, WATERFALL_HEIGHT};

struct KpsApp {
    keys: Arc<Mutex<Vec<KeyState>>>,
    last_frame: std::time::Instant,
}

const BUTTON_WIDTH: f32 = 50.0;
const BUTTON_HEIGHT: f32 = 50.0;
const GAP: f32 = 10.0;

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
                let eased_t = key.update(dt);
                // buttons visuals calculations
                let x = GAP + i as f32 * (BUTTON_WIDTH + GAP);
                let y: f32 = WATERFALL_HEIGHT;
                let base_center = egui::Pos2::new(x + BUTTON_WIDTH / 2.0, y + BUTTON_HEIGHT / 2.0);
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

                // drawing waterfall
                for segment in key.segments.iter() {
                    let segment_x = x;
                    let segment_y = y - segment.y_offset - segment.height;
                    let segment_width = BUTTON_WIDTH;
                    let segment_height = segment.height;

                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::Pos2::new(segment_x, segment_y),
                            egui::vec2(segment_width, segment_height),
                        ),
                        0.0,
                        egui::Color32::WHITE,
                    );
                }
            }
        });
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
