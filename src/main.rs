use eframe::egui::{self, Pos2, Rect, Vec2};
use eframe::epaint::Rgba;
mod keybidings;
use keybidings::KeyBindings;
mod app_visuals_states;
mod application;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true)
            .with_min_inner_size([400.0, 200.0])
            .with_always_on_top()
            .with_resizable(false)
            .with_transparent(true),

        ..Default::default()
    };
    eframe::run_native(
        "Screen Capture",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

#[derive(Debug)]
enum AppState {
    MainApp,
    NewCapture,
    Selection,
    Crop,
    Settings,
}
impl Default for AppState {
    fn default() -> Self {
        AppState::MainApp
    }
}

#[derive(Debug)]
enum TouchedFrame {
    None,
    Bottom,
    Top,
    Right,
    Left,
}

struct MyApp {
    state: AppState,
    selected_area: [Pos2; 2],
    texture: Option<egui::TextureHandle>,
    capture: bool,
    crop: bool,
    image: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    area: bool,
    button_position: Pos2,
    dimensions: Vec2,
    resizing: bool,
    frame: TouchedFrame,
    display_rect: Rect,
    shrink_factor: f32,
    min_pos_top: Pos2,
    key_bindings: KeyBindings,
    delay: u64,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            key_bindings: KeyBindings::new(),
            state: AppState::MainApp,
            button_position: Pos2::new(300.0, 300.0),
            dimensions: Vec2::new(100.0, 100.0),
            resizing: false,
            frame: TouchedFrame::None,
            selected_area: [Pos2::ZERO, Pos2::ZERO],
            texture: None,
            capture: false,
            image: None,
            area: false,
            crop: false,
            display_rect: egui::Rect::ZERO,
            shrink_factor: 0.0,
            min_pos_top: Pos2::ZERO,
            delay: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Rgba::TRANSPARENT.to_rgba_unmultiplied()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_shortcut_press(ctx);
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        let monitor_rect = Rect::from_min_size(Pos2::ZERO, monitor_size);
        match self.state {
            AppState::MainApp => {
                self.main_state_visuals(ctx);
            }
            AppState::NewCapture => {
                self.newcapture_state_visuals(ctx);
            }
            AppState::Selection => {
                self.selection_state_visuals(ctx);
            }
            AppState::Crop => {
                self.crop_state_visuals(ctx, monitor_rect);
            }
            AppState::Settings => {
                self.settings_state_visuals(ctx);
            }
        }
    }
}
