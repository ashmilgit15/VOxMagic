//! VoxMagic - Premium Gold Edition (Final Master)

use cpal::traits::StreamTrait;
use eframe::egui;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::audio::{self, RecordingState};
use crate::settings::AppSettings;
use crate::transcriber::Transcriber;

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_F8, VK_LSHIFT, VK_LWIN, VK_RSHIFT, VK_RWIN,
};

const RELEASE_GRACE_MS: u64 = 250;
const KEY_STATE_IDLE: u8 = 0;
const KEY_STATE_PRESSED: u8 = 1;
const KEY_STATE_RELEASED: u8 = 2;

enum AppMessage {
    TranscriptionStart,
    TranscriptionComplete(String),
    TranscriptionError(String),
}

#[derive(PartialEq, Clone, Copy)]
enum AppState {
    Ready,
    Listening,
    Transcribing,
    Pasting,
}

pub struct VoxMagicApp {
    state: AppState,
    history: Vec<String>,
    status_message: String,

    // UI Animations
    pulse_start: Instant,

    show_settings: bool,
    show_help: bool,
    settings: AppSettings,

    message_rx: Receiver<AppMessage>,
    message_tx: Sender<AppMessage>,

    recording_state: RecordingState,
    active_stream: Option<cpal::Stream>,

    hotkey_state: Arc<AtomicU8>,
    _app_is_running: Arc<AtomicBool>,
    recording_start_time: Option<Instant>,

    clipboard: Option<arboard::Clipboard>,
    enigo: Enigo,
    logo_texture: Option<egui::TextureHandle>,
}

impl VoxMagicApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (message_tx, message_rx) = channel();
        let settings = AppSettings::load();

        let enigo = Enigo::new(&Settings::default()).unwrap_or_else(|_| {
            panic!("Failed to initialize Enigo");
        });

        let hotkey_state = Arc::new(AtomicU8::new(KEY_STATE_IDLE));
        let app_is_running = Arc::new(AtomicBool::new(true));

        let hotkey_state_clone = hotkey_state.clone();
        let app_is_running_clone = app_is_running.clone();

        thread::spawn(move || {
            let mut was_pressed = false;
            let mut release_start: Option<Instant> = None;

            while app_is_running_clone.load(Ordering::Relaxed) {
                unsafe {
                    let f8_down = (GetAsyncKeyState(VK_F8 as i32) as u16 & 0x8000) != 0;
                    let lshift_down = (GetAsyncKeyState(VK_LSHIFT as i32) as u16 & 0x8000) != 0;
                    let rshift_down = (GetAsyncKeyState(VK_RSHIFT as i32) as u16 & 0x8000) != 0;
                    let lwin_down = (GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0;
                    let rwin_down = (GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0;

                    let is_pressed = f8_down || ((lshift_down || rshift_down) && (lwin_down || rwin_down));

                    if is_pressed {
                        release_start = None;
                        if !was_pressed {
                            hotkey_state_clone.store(KEY_STATE_PRESSED, Ordering::SeqCst);
                            was_pressed = true;
                        }
                    } else if was_pressed {
                        if release_start.is_none() {
                            release_start = Some(Instant::now());
                        }

                        if let Some(start) = release_start {
                            if start.elapsed().as_millis() >= RELEASE_GRACE_MS as u128 {
                                hotkey_state_clone.store(KEY_STATE_RELEASED, Ordering::SeqCst);
                                was_pressed = false;
                                release_start = None;
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            state: AppState::Ready,
            history: Vec::new(),
            status_message: if settings.groq_api_key.is_empty() { "⚠️ Setup Required" } else { "Ready" }.to_string(),
            pulse_start: Instant::now(),
            show_settings: settings.groq_api_key.is_empty(),
            show_help: settings.groq_api_key.is_empty(),
            settings,
            message_rx,
            message_tx,
            recording_state: RecordingState::new(),
            active_stream: None,
            hotkey_state,
            _app_is_running: app_is_running,
            recording_start_time: None,
            clipboard: arboard::Clipboard::new().ok(),
            enigo,
            logo_texture: None,
        }
    }

    fn start_recording(&mut self) {
        if self.state != AppState::Ready || self.settings.groq_api_key.is_empty() {
            if self.settings.groq_api_key.is_empty() { self.show_settings = true; }
            return;
        }

        match audio::start_recording(&self.recording_state) {
            Ok(stream) => {
                self.active_stream = Some(stream);
                self.state = AppState::Listening;
                self.recording_start_time = Some(Instant::now());
                self.status_message = "Listening...".to_string();
            }
            Err(e) => {
                self.status_message = format!("Mic Error: {}", e);
            }
        }
    }

    fn stop_recording(&mut self) {
        if self.state != AppState::Listening { return; }

        if let Some(stream) = self.active_stream.take() {
            let _ = stream.pause();
            drop(stream);
        }

        self.state = AppState::Transcribing;
        self.status_message = "Refining...".to_string();

        let audio_result = audio::stop_recording(&self.recording_state);
        let tx = self.message_tx.clone();
        let api_key = self.settings.groq_api_key.clone();

        thread::spawn(move || {
            match audio_result {
                Ok(audio_data) => {
                    let _ = tx.send(AppMessage::TranscriptionStart);
                    let transcriber = Transcriber::new(api_key);
                    match transcriber.transcribe(audio_data) {
                        Ok(result) => {
                            let _ = tx.send(AppMessage::TranscriptionComplete(result.text));
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::TranscriptionError(format!("{}", e)));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::TranscriptionError(format!("{}", e)));
                }
            }
        });
    }

    fn process_hotkey(&mut self) {
        let state = self.hotkey_state.load(Ordering::SeqCst);
        match state {
            KEY_STATE_PRESSED => {
                self.hotkey_state.store(KEY_STATE_IDLE, Ordering::SeqCst);
                self.start_recording();
            }
            KEY_STATE_RELEASED => {
                self.hotkey_state.store(KEY_STATE_IDLE, Ordering::SeqCst);
                self.stop_recording();
            }
            _ => {}
        }
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                AppMessage::TranscriptionStart => { self.state = AppState::Transcribing; }
                AppMessage::TranscriptionComplete(text) => {
                    let cleaned_text = text.trim().to_string();
                    if !cleaned_text.is_empty() {
                        // FIX DOUBLING: History log is read-only
                        self.history.insert(0, cleaned_text.clone());
                        if self.history.len() > 10 { self.history.pop(); }

                        if self.settings.auto_paste {
                            self.state = AppState::Pasting;
                            self.status_message = "Pasting...".to_string();
                            self.paste_text_to_active_window(&cleaned_text);
                        }

                        self.status_message = "Success".to_string();
                    } else {
                        self.status_message = "No speech detected".to_string();
                    }
                    self.state = AppState::Ready;
                }
                AppMessage::TranscriptionError(error) => {
                    self.status_message = format!("Error: {}", error);
                    self.state = AppState::Ready;
                }
            }
        }
    }

    fn paste_text_to_active_window(&mut self, text: &str) {
        let start = Instant::now();
        unsafe {
            while start.elapsed() < Duration::from_millis(800) {
                let lwin = (GetAsyncKeyState(VK_LWIN as i32) as u16 & 0x8000) != 0;
                let rwin = (GetAsyncKeyState(VK_RWIN as i32) as u16 & 0x8000) != 0;
                let lshift = (GetAsyncKeyState(VK_LSHIFT as i32) as u16 & 0x8000) != 0;
                let rshift = (GetAsyncKeyState(VK_RSHIFT as i32) as u16 & 0x8000) != 0;
                if !lwin && !rwin && !lshift && !rshift { break; }
                thread::sleep(Duration::from_millis(20));
            }
        }

        if let Some(ref mut clipboard) = self.clipboard {
            if clipboard.set_text(text.to_string()).is_ok() {
                thread::sleep(Duration::from_millis(150));
                let _ = self.enigo.key(Key::Control, Direction::Press);
                thread::sleep(Duration::from_millis(30));
                let _ = self.enigo.key(Key::Unicode('v'), Direction::Click);
                thread::sleep(Duration::from_millis(30));
                let _ = self.enigo.key(Key::Control, Direction::Release);
            }
        }
    }
}

impl eframe::App for VoxMagicApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_hotkey();
        self.process_messages();
        ctx.request_repaint_after(Duration::from_millis(16));

        // Always on Top
        let level = if self.settings.always_on_top { egui::WindowLevel::AlwaysOnTop } else { egui::WindowLevel::Normal };
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));

        // Styling
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = 24.0.into();
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(7, 7, 10)).inner_margin(28.0))
            .show(ctx, |ui| {

                // Header
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("VOXMAGIC").size(11.0).strong().extra_letter_spacing(2.0).color(egui::Color32::from_rgb(168, 85, 247)));
                        ui.add_space(-4.0);
                        ui.label(egui::RichText::new("Gold Edition").size(24.0).strong().color(egui::Color32::WHITE));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new("⚙").size(20.0)).clicked() {
                            self.show_settings = !self.show_settings;
                            if self.show_settings { self.show_help = false; }
                        }
                        ui.add_space(8.0);
                        if ui.button(egui::RichText::new("❓").size(18.0)).clicked() {
                            self.show_help = !self.show_help;
                            if self.show_help { self.show_settings = false; }
                        }
                    });
                });

                ui.add_space(20.0);

                // --- ONBOARDING GUIDE ---
                if self.show_help {
                    egui::Frame::none().fill(egui::Color32::from_rgb(15, 15, 25)).rounding(12.0).inner_margin(16.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 60))).show(ui, |ui| {
                        ui.label(egui::RichText::new("GETTING STARTED").size(10.0).strong().color(egui::Color32::from_rgb(168, 85, 247)));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("1. Get your API Key").strong());
                        ui.horizontal(|ui| {
                            ui.label("Visit");
                            ui.hyperlink_to("Groq Console", "https://console.groq.com/keys");
                            ui.label("to get a free key.");
                        });
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("2. Setup").strong());
                        ui.label("Click ⚙, paste your key, and enable Always on Top.");
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new("3. Magic").strong());
                        ui.label("Hold F8, speak, and let go to paste refined text!");
                        ui.add_space(10.0);
                        if ui.button("Start Flowing").clicked() { self.show_help = false; }
                    });
                    ui.add_space(15.0);
                }

                // --- SETTINGS ---
                if self.show_settings {
                    egui::Frame::none().fill(egui::Color32::from_rgb(18, 18, 24)).rounding(16.0).inner_margin(20.0).stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 40, 55))).show(ui, |ui| {
                        ui.label(egui::RichText::new("CONFIG").size(10.0).strong().color(egui::Color32::from_rgb(100, 100, 120)));
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            ui.label("Groq Key:");
                            let resp = ui.add(egui::TextEdit::singleline(&mut self.settings.groq_api_key).password(true));
                            if resp.changed() { let _ = self.settings.save(); }
                        });
                        ui.add_space(10.0);
                        if ui.checkbox(&mut self.settings.auto_paste, "Magic Auto-Paste").changed() { let _ = self.settings.save(); }
                        if ui.checkbox(&mut self.settings.always_on_top, "Always on Top").changed() { let _ = self.settings.save(); }
                    });
                    ui.add_space(20.0);
                }

                // --- PULSE ---
                ui.vertical_centered(|ui| {
                    // Lazy load texture from embedded bytes
                    if self.logo_texture.is_none() {
                        let image_data = include_bytes!("../VoxMagicLogo.png");
                        if let Ok(image) = image::load_from_memory(image_data) {
                            let rgba = image.to_rgba8();
                            let size = [rgba.width() as _, rgba.height() as _];
                            let pixels = rgba.into_raw();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                            self.logo_texture = Some(ui.ctx().load_texture("logo", color_image, Default::default()));
                        }
                    }

                    let time = self.pulse_start.elapsed().as_secs_f32();
                    let (size, color) = match self.state {
                        AppState::Listening => (70.0 + (time * 8.0).sin() * 10.0, egui::Color32::from_rgb(168, 85, 247)),
                        AppState::Transcribing | AppState::Pasting => (60.0, egui::Color32::from_rgb(59, 130, 246)),
                        AppState::Ready => (50.0, egui::Color32::from_rgb(30, 30, 40)),
                    };
                    
                    let (rect, _) = ui.allocate_at_least(egui::vec2(ui.available_width(), 120.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), size, color);

                    if let Some(texture) = &self.logo_texture {
                        let img_size = size * 0.7; // Logo slightly smaller than pulse
                        let img_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(img_size, img_size));
                        ui.painter().image(texture.id(), img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
                    } else {
                        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, match self.state {
                            AppState::Listening => "🎙", AppState::Transcribing => "🧠", AppState::Pasting => "⚡", AppState::Ready => "✨",
                        }, egui::FontId::proportional(24.0), egui::Color32::WHITE);
                    }
                });

                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(&self.status_message).size(16.0).strong().color(egui::Color32::from_rgb(200, 200, 210)));
                });

                ui.add_space(30.0);

                // --- LOG ---
                ui.label(egui::RichText::new("RECENT FLOWS").size(10.0).strong().color(egui::Color32::from_rgb(100, 100, 120)));
                ui.add_space(10.0);
                egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                    for entry in &self.history {
                        egui::Frame::none().fill(egui::Color32::from_rgb(12, 12, 16)).rounding(12.0).inner_margin(14.0).show(ui, |ui| {
                            ui.add(egui::Label::new(egui::RichText::new(entry).size(14.0).color(egui::Color32::from_rgb(180, 180, 190))).wrap());
                        });
                        ui.add_space(10.0);
                    }
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("🗑 Clear").clicked() { self.history.clear(); }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new("HOLD F8 TO COMMENCE").size(10.0).strong().color(egui::Color32::from_rgb(80, 80, 100)));
                    });
                });
            });
    }
}
