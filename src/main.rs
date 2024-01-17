use eframe::egui::panel::TopBottomSide;
use eframe::egui::{
    self, pos2, Button, CentralPanel, Event, Frame, Id, Key, KeyboardShortcut, Layout, Modifiers,
    Pos2, Rect, Sense, Style, TopBottomPanel, Ui, Vec2, ViewportCommand,
};
use eframe::emath::Align;
use eframe::epaint::{vec2, Color32, Rgba, Rounding, Stroke};
use egui::text::LayoutJob;
use image::{codecs::gif::GifEncoder, imageops};
use rfd::FileDialog;
use screenshots::Screen;
use std::fs::OpenOptions;
use std::ops::{Add, Div};
use struct_iterable::Iterable;

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
    eframe::run_native("Screen", options, Box::new(|_cc| Box::<MyApp>::default()))
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
#[derive(Iterable)]
struct KeyBindings {
    save: egui::Key,
    cancel: egui::Key,
    new: egui::Key,
    crop: egui::Key,
    fullscreen: egui::Key,
    // Add more actions as needed
}
impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            save: egui::Key::S,
            cancel: egui::Key::Z,
            new: egui::Key::N,
            crop: egui::Key::X,
            fullscreen: egui::Key::F,
        }
    }
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
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            key_bindings: KeyBindings::default(),
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
                //Navbar
                egui::TopBottomPanel::top("buttons navbar").show(ctx, |ui| {
                    //Organize buttons in horizontal navbar
                    ui.horizontal(|ui| {
                        if ui.button("New capture").clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::ZERO));
                            ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                            //Workaroud for borderless fullscreen transparency bug
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(
                                monitor_size.add(Vec2::new(1.0, 1.0)),
                            ));
                            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                                egui::WindowLevel::AlwaysOnTop,
                            ));
                            ctx.send_viewport_cmd(ViewportCommand::Focus);
                            self.state = AppState::NewCapture;
                        }
                        if ui.button("Settings").clicked() {
                            self.state = AppState::Settings;
                        }
                    });
                });

                CentralPanel::default().show(ctx, |ui| {
                    if let Some(texture) = self.texture.clone() {
                        egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                            let selection = egui::Rect::from_two_pos(
                                self.selected_area[0],
                                self.selected_area[1],
                            );

                            let uv = egui::Rect::from_two_pos(
                                Pos2::new(
                                    selection.left_top().x / monitor_size.x,
                                    selection.left_top().y / monitor_size.y,
                                ),
                                Pos2::new(
                                    selection.right_bottom().x / monitor_size.x,
                                    selection.right_bottom().y / monitor_size.y,
                                ),
                            );

                            ui.allocate_ui_with_layout(
                                monitor_size.add(vec2(20.0, 20.0)),
                                Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    let a =
                                        ui.allocate_exact_size(selection.size(), Sense::click());
                                    ui.painter().image(texture.id(), a.0, uv, Color32::WHITE);
                                },
                            );

                            egui::Window::new("options")
                                .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
                                .collapsible(false)
                                .resizable(false)
                                .title_bar(false)
                                .show(ctx, |ui| {
                                    //Organize buttons in horizontal line
                                    ui.horizontal(|ui| {
                                        if ui.button("Crop").clicked() {
                                            let a = egui::Rect::from_two_pos(
                                                self.selected_area[0],
                                                self.selected_area[1],
                                            );

                                            self.button_position = a.min;
                                            self.crop = true;
                                            self.dimensions = egui::vec2(a.width(), a.height());
                                            self.state = AppState::Crop;
                                        }

                                        if ui.button("Save").clicked() {
                                            self.save_capture(ctx);
                                        }
                                    });
                                });
                        });
                    }
                });
            }
            AppState::NewCapture => {
                let pointer: egui::PointerState = ctx.input(|i| i.pointer.clone());

                // Very little opacity for this frame, only to show area that its possible to capture
                let semi_transparent_frame = Frame::none().fill(Color32::WHITE.gamma_multiply(0.1));

                CentralPanel::default()
                    .frame(semi_transparent_frame)
                    .show(ctx, |ui| {
                        // Make pointer into crosshair if its hovering the selection area, if its on window pointer stays classic
                        if ui.ui_contains_pointer() {
                            ctx.output_mut(|o| {
                                o.cursor_icon = egui::CursorIcon::Crosshair;
                            })
                        }
                    });
                // If button has been pressed dont show this window
                if !self.area {
                    egui::Window::new("options")
                        .anchor(egui::Align2::CENTER_TOP, [0.0, 0.0])
                        .collapsible(false)
                        .resizable(false)
                        .title_bar(false)
                        .show(ctx, |ui| {
                            //Organize buttons in horizontal line
                            ui.horizontal(|ui| {
                                if ui.button("Full screen").clicked() {
                                    let monitor_size: Vec2 =
                                        ctx.input(|i| i.viewport().monitor_size.unwrap());
                                    let monitor_rect =
                                        Rect::from_min_size(Pos2::ZERO, monitor_size);
                                    // Store full screen selection
                                    self.selected_area[0] = Pos2::ZERO;
                                    self.selected_area[1] = monitor_rect.right_bottom();
                                    //Go to Selection state
                                    self.state = AppState::Selection;

                                    // This makes it skip selection of second point in selection state of app
                                    self.capture = true;

                                    //Request repaint in order to wait until window is transparent
                                    ctx.request_repaint();
                                }

                                if ui.button("Area").clicked() {
                                    self.area = true;
                                }
                            });
                            // Selection if button has been pressed, must do it this way otherwise button click is recorded as first point of selection
                            if pointer.primary_clicked() && !ui.ui_contains_pointer() {
                                self.area = true;
                                self.selected_area[0] =
                                    ctx.input(|i| i.pointer.interact_pos().unwrap());
                                self.state = AppState::Selection;
                            }
                        });
                }
                // Selection if button has been pressed
                if pointer.primary_clicked() && self.area {
                    self.area = true;
                    self.selected_area[0] = ctx.input(|i| i.pointer.interact_pos().unwrap());
                    self.state = AppState::Selection;
                }
            }

            AppState::Selection => {
                //reset option window in type of selection
                self.area = false;

                let transparent_frame = Frame::none().fill(Color32::TRANSPARENT);
                CentralPanel::default()
                    .frame(transparent_frame)
                    .show(ctx, |ui| {
                        //Make pointer into crosshair
                        if ui.ui_contains_pointer() {
                            ctx.output_mut(|o| {
                                o.cursor_icon = egui::CursorIcon::Crosshair;
                            });
                        }
                        //Check for pointer changes
                        let pointer = ctx.input(|i| i.pointer.clone());
                        if pointer.is_decidedly_dragging() && !self.capture {
                            let pointer_pos = pointer.hover_pos().unwrap();
                            let rect = egui::Rect::from_two_pos(
                                self.selected_area[0],
                                pointer.hover_pos().unwrap(),
                            );
                            ui.painter().rect_stroke(
                                rect,
                                Rounding::ZERO,
                                Stroke::new(1.0, Color32::RED),
                            );
                            if pointer.primary_released() {
                                self.selected_area[1] = pointer_pos;
                                ctx.request_repaint();
                                self.capture = true;
                            }
                        }
                        if self.capture && !ctx.has_requested_repaint() {
                            let screens = Screen::all().unwrap();

                            //Possible to change screen to capture
                            let primary_screen = screens[0];
                            self.image = Some(primary_screen.capture().unwrap());
                            self.capture = false;

                            let image = self.image.clone().unwrap();
                            // Conversion of screnshoots crate img to egui renderable img
                            let pixels: Vec<Color32> = image
                                .pixels()
                                .map(|pixel| {
                                    Color32::from_rgba_unmultiplied(
                                        pixel.0[0], pixel.0[1], pixel.0[2], pixel.0[3],
                                    )
                                })
                                .collect();

                            let img = egui::ColorImage {
                                pixels: pixels,
                                size: [image.width() as usize, image.height() as usize],
                            };
                            //Store texture of screenshot in MainApp
                            self.texture =
                                Some(ui.ctx().load_texture("screenshot", img, Default::default()));

                            // Reset window
                            ctx.send_viewport_cmd(ViewportCommand::Decorations(true));
                            ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
                            ctx.send_viewport_cmd(ViewportCommand::Focus);
                            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                                egui::WindowLevel::Normal,
                            ));

                            //Change state to Main state
                            self.state = AppState::MainApp;
                        }
                    });
            }

            AppState::Crop => {
                egui::TopBottomPanel::top("buttons navbar").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("CANCEL").clicked() {
                            self.state = AppState::MainApp;
                        }
                        if ui.button("CONFIRM").clicked() {
                            self.button_position.x = (self.button_position.x
                                - self.display_rect.left_top().x)
                                / self.shrink_factor;
                            self.button_position.y = (self.button_position.y
                                - self.display_rect.left_top().y)
                                / self.shrink_factor;
                            self.selected_area[0] = self.button_position;
                            self.dimensions = self.dimensions.div(self.shrink_factor);
                            self.selected_area[1] = self.selected_area[0].add(self.dimensions);
                            self.state = AppState::MainApp;
                        }
                    });
                });

                CentralPanel::default().show(ctx, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink(true)
                        .drag_to_scroll(true)
                        .show(ui, |ui| {
                            if let Some(texture) = self.texture.as_ref() {
                                let uv = egui::Rect::from_two_pos(Pos2::ZERO, pos2(1.0, 1.0));

                                let avheight =
                                    ui.available_rect_before_wrap().shrink(60.0).height();
                                let avwidth = avheight * monitor_rect.aspect_ratio();

                                let rect = Rect::from_center_size(
                                    ui.available_rect_before_wrap().center(),
                                    vec2(avwidth, avheight),
                                );
                                println!(
                                    "uv: {:?}, avheight: {:?}, avwidth: {:?}, rect: {:?}",
                                    uv, avheight, avwidth, rect
                                );
                                if self.crop {
                                    self.shrink_factor = avwidth / monitor_rect.width();
                                    let selected_area =
                                        Rect::from_center_size(Pos2::ZERO, self.dimensions);
                                    let new_w = self.dimensions.x * self.shrink_factor;
                                    let new_h = (self.dimensions.x * self.shrink_factor)
                                        / selected_area.aspect_ratio();
                                    let new_x = self.button_position.x * self.shrink_factor;
                                    let new_y = self.button_position.y * self.shrink_factor;

                                    self.min_pos_top = rect.left_top();
                                    self.button_position = rect.left_top() + vec2(new_x, new_y); // new button = rect.left_top.x + original.x * self.shrink_factor
                                    self.dimensions = vec2(new_w, new_h);
                                    self.display_rect = rect;
                                    self.crop = false;
                                }
                                ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
                            }
                        });
                });

                CentralPanel::default()
                    .frame(Frame::none().fill(Color32::TRANSPARENT))
                    .show(ctx, |ui| {
                        // Draw the button element
                        let pos = self.button_position.clone();
                        let dimensions = self.dimensions.clone();
                        MyApp::drag(self, ui, ui.id(), |ui| {
                            let rect = Rect::from_min_size(pos, dimensions);
                            ui.put(
                                rect,
                                Button::new("")
                                    .fill(Color32::TRANSPARENT)
                                    .sense(Sense::click()),
                            );
                        });
                    });
            }
            AppState::Settings => {
                TopBottomPanel::new(TopBottomSide::Top, "back").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        // Add a heading with text "Settings" (big)

                        ui.heading(egui::RichText::new("Settings: ").size(30.0));
                        // Add a button on the right corner
                        ui.add_space(ui.available_size().x - 0.0);

                        if ui.button("Go back").clicked() {
                            self.state = AppState::Selection;
                        }
                    });
                });
                CentralPanel::default().show(ctx, |ui| {
                    ui.label(
                        "Modify keybidings by hovering on key you want to modify and pressing new desidered key on keyboard"
                    );
                    ui.horizontal(|ui| {
                        ui.label("New fullscreen capture: ");
                        ui.add_enabled(false, Button::new("Ctrl"));
                        ui.label("+");
                        if ui.button(format!("{:?}", self.key_bindings.fullscreen)).hovered() {
                            ui.input(|i| {
                                for key in Key::ALL {
                                    if
                                        i.key_pressed(key.to_owned()) &&
                                        !self.is_key_assigned(key.to_owned())
                                    {
                                        self.key_bindings.fullscreen = key.to_owned();
                                    }
                                }
                            })
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("New capture: ");
                        ui.add_enabled(false, Button::new("Ctrl"));
                        ui.label("+");
                        if ui.button(format!("{:?}", self.key_bindings.new)).hovered() {
                            ui.input(|i| {
                                for key in Key::ALL {
                                    if
                                        i.key_pressed(key.to_owned()) &&
                                        !self.is_key_assigned(key.to_owned())
                                    {
                                        self.key_bindings.new = key.to_owned();
                                    } else {
                                    }
                                }
                            })
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Save capture: ");
                        ui.add_enabled(false, Button::new("Ctrl"));
                        ui.label("+");
                        if ui.button(format!("{:?}", self.key_bindings.save)).hovered() {
                            ui.input(|i| {
                                for key in Key::ALL {
                                    if
                                        i.key_pressed(key.to_owned()) &&
                                        !self.is_key_assigned(key.to_owned())
                                    {
                                        self.key_bindings.save = key.to_owned();
                                    }
                                }
                            })
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Cancel cropping: ");
                        ui.add_enabled(false, Button::new("Ctrl"));
                        ui.label("+");
                        if ui.button(format!("{:?}", self.key_bindings.cancel)).hovered() {
                            let input = ui.input(|i| i.clone());
                            for key in Key::ALL {
                                if
                                    input.key_pressed(key.to_owned()) &&
                                    !self.is_key_assigned(key.to_owned())
                                {
                                    self.key_bindings.cancel = key.to_owned();
                                }
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Resize crop: ");
                        ui.add_enabled(false, Button::new("Ctrl"));
                        ui.label("+");
                        if ui.button(format!("{:?}", self.key_bindings.crop)).hovered() {
                            ui.input(|i| {
                                for key in Key::ALL {
                                    if
                                        i.key_pressed(key.to_owned()) &&
                                        !self.is_key_assigned(key.to_owned())
                                    {
                                        self.key_bindings.crop = key.to_owned();
                                    }
                                }
                            })
                        }
                    });
                });
            }
        }
    }
}

impl MyApp {
    fn check_shortcut_press(&self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());
        input.events.iter().for_each(|event| {
            println!("{:?}", event);
            println!("{:?}", self.key_bindings.save);
            match event.to_owned() {
                Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat,
                    modifiers,
                } => {
                    if key == self.key_bindings.save && modifiers.ctrl && !repeat && pressed {
                        self.save_capture(ctx);
                    }
                }
                _ => {}
            }
        });
    }
    fn is_key_assigned(&self, key: Key) -> bool {
        // Check if the key is already assigned to any action
        self.key_bindings.fullscreen == key
            || self.key_bindings.new == key
            || self.key_bindings.save == key
            || self.key_bindings.cancel == key
            || self.key_bindings.crop == key
    }
    fn save_capture(&self, ctx: &egui::Context) {
        if let Some(texture) = self.texture.clone() {
            let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
            let monitor_rect = Rect::from_min_size(Pos2::ZERO, monitor_size);
            let selection = egui::Rect::from_two_pos(self.selected_area[0], self.selected_area[1]);
            let files = FileDialog::new()
                .add_filter("PNG", &["png"])
                .add_filter("JPG", &["jpg"])
                .add_filter("GIF", &["gif"])
                .set_file_name("screenshot")
                .set_directory("/")
                .save_file();
            let ext = files.clone();
            if let Some(mut img) = self.image.clone() {
                let shrink = monitor_rect.width() / (img.width() as f32);
                let top_left_x = (selection.left_top().x - Pos2::ZERO.x) / shrink;
                let top_left_y = (selection.left_top().y - Pos2::ZERO.y) / shrink;
                let width = selection.width() / shrink;
                let height = selection.height() / shrink;

                let img_crop = imageops::crop(
                    &mut img,
                    top_left_x as u32,
                    top_left_y as u32,
                    width as u32,
                    height as u32,
                );

                let img = img_crop.to_image();
                if let Some(save_path) = ext.as_ref() {
                    if let Some(extension) = save_path.extension() {
                        let extension_str = extension.to_string_lossy().to_lowercase();
                        match extension.to_str() {
                            Some("jpg") | Some("jpeg") | Some("png") => {
                                img.save(files.unwrap().as_path()).unwrap();
                            }
                            Some("gif") => {
                                let file = OpenOptions::new()
                                    .create(true)
                                    .read(true)
                                    .write(true)
                                    .open(save_path.as_path())
                                    .unwrap();

                                let frame = image::Frame::new(img);

                                let mut encoder = GifEncoder::new_with_speed(file, 30);
                                encoder.encode_frame(frame).unwrap();
                            }
                            _ => println!("Unsupported file extension: {}", extension_str),
                        }
                    }
                }
            }
        }
    }

    pub fn drag(&mut self, ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
        let response = ui.scope(body).response;
        let response = ui.interact(response.rect, id, Sense::drag());
        let mut min = response.rect.right_bottom();
        min.x -= self.dimensions.x;
        min.y -= self.dimensions.y;

        let outline = Rect::from_min_size(min, self.dimensions);

        let mut resize_frame = outline.shrink2(Vec2::new(8.0, 8.0));
        resize_frame.set_center(outline.center());
        let dx = Rect::from_two_pos(resize_frame.right_bottom(), outline.right_top());
        let top = Rect::from_two_pos(resize_frame.right_top(), outline.left_top());
        let down = Rect::from_two_pos(resize_frame.right_bottom(), outline.left_bottom());
        let sx = Rect::from_two_pos(resize_frame.left_top(), outline.left_bottom());

        ui.painter()
            .rect_stroke(outline, 0.0, Stroke::new(3.0, Color32::WHITE));
        if response.hovered()
            && ui.rect_contains_pointer(outline)
            && !ui.rect_contains_pointer(resize_frame)
        {
            ui.output_mut(|o| {
                o.cursor_icon = egui::CursorIcon::Crosshair;
            });
        }
        if response.drag_started()
            && ui.rect_contains_pointer(outline)
            && !ui.rect_contains_pointer(resize_frame)
        {
            if ui.rect_contains_pointer(down) {
                self.frame = TouchedFrame::Bottom;
            } else if ui.rect_contains_pointer(top) {
                self.frame = TouchedFrame::Top;
            } else if ui.rect_contains_pointer(dx) {
                self.frame = TouchedFrame::Right;
            } else if ui.rect_contains_pointer(sx) {
                self.frame = TouchedFrame::Left;
            }
            self.resizing = true;
            ui.output_mut(|o| {
                o.cursor_icon = egui::CursorIcon::None;
            });
        }
        if self.resizing && response.dragged() {
            match self.frame {
                TouchedFrame::Top => {
                    if !(self.dimensions.y - response.drag_delta().y < 15.0
                        || self.button_position.y + response.drag_delta().y
                            < self.display_rect.left_top().y)
                    {
                        self.button_position.y += response.drag_delta().y;
                        self.dimensions.y -= response.drag_delta().y;
                    }
                }
                TouchedFrame::Bottom => {
                    if !(self.dimensions.y + response.drag_delta().y < 15.0
                        || self.button_position.y + self.dimensions.y + response.drag_delta().y
                            > self.display_rect.right_bottom().y)
                    {
                        self.dimensions.y += response.drag_delta().y;
                    }
                }
                TouchedFrame::Right => {
                    if !(self.dimensions.x + response.drag_delta().x < 15.0
                        || self.button_position.x + self.dimensions.x + response.drag_delta().x
                            > self.display_rect.right_top().x)
                    {
                        self.dimensions.x += response.drag_delta().x;
                    }
                }
                TouchedFrame::Left => {
                    if !(self.dimensions.x - response.drag_delta().x < 15.0
                        || self.button_position.x + response.drag_delta().x
                            < self.display_rect.left_top().x)
                    {
                        self.button_position.x += response.drag_delta().x;
                        self.dimensions.x -= response.drag_delta().x;
                    }
                }
                _ => {}
            }
        }
        if response.drag_released() {
            self.resizing = false;
        }
    }
}
