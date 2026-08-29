mod key_state;

use eframe::egui::{self};
use evdev::EventSummary;
use std::sync::{Arc, Mutex};

use key_state::{KEY_DOWN, KEY_UP, KEYS, KeyState, ease_out};
use crate::key_state::Segment;

struct KpsApp {
    keys: Arc<Mutex<Vec<KeyState>>>,
    last_frame: std::time::Instant,
}

const BUTTON_WIDTH: f32 = 50.0;
const BUTTON_HEIGHT: f32 = 50.0;
const GAP: f32 = 10.0;
const RISE_TIME: f32 = 0.01;
const FALL_TIME: f32 = 0.2;

const WATERFALL_SPEED: f32 = 300.0;
const WATERFALL_HEIGHT: f32 = 500.0;

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
                let just_pressed = key.is_down && !key.was_down;
                let just_released = !key.is_down && key.was_down;

                // buttons visuals calculations
                if key.is_down {
                    key.t = (key.t + dt / RISE_TIME).min(1.0);
                } else {
                    key.t = (key.t - dt / FALL_TIME).max(0.0);
                }
                let eased_t = ease_out(key.t);

                let x = GAP + i as f32 * (BUTTON_WIDTH + GAP);
                let y: f32 = 10.0 + WATERFALL_HEIGHT;
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

                // waterfall visuals
                if just_pressed {
                    key.segments.push(Segment { height: 0.0, y_offset: 0.0, growing: true });
                    key.press_count += 1;
                }
                if just_released {
                    if let Some(seg) = key.segments.iter_mut().find(|s| s.growing) {
                        seg.growing = false;
                    }
                }

                // growing
                if let Some(last) = key.segments.last_mut() {
                    if last.growing {
                        last.height += WATERFALL_SPEED * dt;
                        last.y_offset -= WATERFALL_SPEED * dt;
                    }
                }

                // offset waterfall
                for segment in key.segments.iter_mut() {
                    segment.y_offset += WATERFALL_SPEED * dt;
                }

                key.segments.retain(|s| s.y_offset < WATERFALL_HEIGHT); // cleanup

                key.was_down = key.is_down;

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
