use super::MyApp;
use eframe::egui::{
    self, Event, Id, Pos2, Rect, Sense, Ui,
    Vec2, ViewportCommand
};
use eframe::epaint::{ Color32,  Stroke};
use image::{codecs::gif::GifEncoder, imageops};
use rfd::FileDialog;
use std::borrow::Cow;
use std::fs::OpenOptions;
use std::ops::Add;

use super::TouchedFrame;
use super::AppState;

impl MyApp {
    pub fn copy_to_clipboard(&self, ctx: &egui::Context) {
        if let Some(_texture) = self.texture.clone() {
            let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = self.crop_image(ctx);
            let a = img.clone().into_raw();
            let img_to_save = arboard::ImageData {
                width: img.width() as usize,
                height: img.height() as usize,
                bytes: Cow::from(a),
            };
            let mut clipboard = arboard::Clipboard::new().unwrap();
            clipboard.set_image(img_to_save).unwrap();
        }
    }
    //------ Calculates dimensions, center of rectangle where image is going to rendered
    pub fn calculate_space(&self, ctx: &egui::Context, ui: &mut Ui) -> Rect {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        let selection = egui::Rect::from_two_pos(self.selected_area[0], self.selected_area[1]);
        let center = Pos2::new(monitor_size.x / 2.0, monitor_size.y / 2.0);
        let mut space = egui::Rect::from_center_size(center, selection.size());

        //Modify space in order to render image clearly and not stretched
        if !ui.available_rect_before_wrap().contains_rect(space) {
            let available_width = ui.available_rect_before_wrap().width();
            let available_height = ui.available_rect_before_wrap().height();
            let aspect_ratio = selection.aspect_ratio();
            let new_width = available_height * aspect_ratio;
            if new_width <= available_width {
                ui.set_width(new_width);
                space = ui.available_rect_before_wrap();
                space.set_center(Pos2::new(center.x, space.center().y));
            } else {
                let new_height = available_width / aspect_ratio;
                ui.set_height(new_height);
                space = ui.available_rect_before_wrap();
                space.set_center(Pos2::new(space.center().x, center.y));
            }
        }
        return space;
    }

    //-----Calculates area of image to render, aka part of image selected by user
    pub fn calculate_uv(&self, ctx: &egui::Context) -> Rect {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());

        let selection = egui::Rect::from_two_pos(self.selected_area[0], self.selected_area[1]);

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
        return uv;
    }
    //------Sets the window to optimal configuration for screen capture
    pub fn set_new_capture_window(&mut self, ctx: &egui::Context) {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::ZERO));
        ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(
            monitor_size.add(Vec2::new(1.0, 1.0)),
        ));
        /*         ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        )); */
        ctx.send_viewport_cmd(ViewportCommand::Focus);
    }
    //------Checks if any shortcut has been pressed
    pub fn check_shortcut_press(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());
        input
            .events
            .iter()
            .for_each(|event| match event.to_owned() {
                Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    repeat,
                    modifiers,
                } => {
                    if key == self.key_bindings.save && modifiers.ctrl && !repeat && pressed {
                        self.save_capture(ctx);
                    } else if key == self.key_bindings.cancel
                        && modifiers.ctrl
                        && !repeat
                        && pressed
                        && matches!(self.state, AppState::Crop)
                    {
                        self.delay = 0;
                        self.state = AppState::MainApp;
                    } else if key == self.key_bindings.fullscreen
                        && modifiers.ctrl
                        && !repeat
                        && pressed
                        && matches!(self.state, AppState::MainApp)
                    {
                        self.delay = 0;
                        self.set_new_capture_window(ctx);
                        self.handle_fullscreen_capture(ctx);
                    } else if key == self.key_bindings.new
                        && modifiers.ctrl
                        && !repeat
                        && pressed
                        && matches!(self.state, AppState::MainApp)
                    {
                        self.delay = 0;
                        self.set_new_capture_window(ctx);
                        self.area = true;
                        self.state = AppState::NewCapture;
                    } else if key == self.key_bindings.crop
                        && modifiers.ctrl
                        && !repeat
                        && matches!(self.state, AppState::MainApp)
                    {
                        self.delay = 0;
                        self.handle_crop_request(ctx);
                    } else if key == self.key_bindings.clipboard
                        && modifiers.ctrl
                        && !repeat
                        && matches!(self.state, AppState::MainApp)
                    {
                        self.delay = 0;
                        self.copy_to_clipboard(ctx);
                    }
                }
                _ => {}
            });
    }
    //--------
    pub fn handle_crop_request(&mut self, _ctx: &egui::Context) {
        if let Some(_texture) = self.texture.clone() {
            let a = egui::Rect::from_two_pos(self.selected_area[0], self.selected_area[1]);
            self.button_position = a.min;
            self.crop = true;
            self.dimensions = egui::vec2(a.width(), a.height());
            self.state = AppState::Crop;
        }
    }
    pub fn handle_fullscreen_capture(&mut self, ctx: &egui::Context) {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        let monitor_rect = Rect::from_min_size(Pos2::ZERO, monitor_size);
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
    fn crop_image(&self, ctx: &egui::Context) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        let monitor_rect = Rect::from_min_size(Pos2::ZERO, monitor_size);
        let selection = egui::Rect::from_two_pos(self.selected_area[0], self.selected_area[1]);
        let mut img = self.image.clone().unwrap();
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

        img_crop.to_image()
    }
    pub fn save_capture(&self, ctx: &egui::Context) {
        if !matches!(self.state, AppState::MainApp) {
            return;
        }
        if let Some(_texture) = self.texture.clone() {
            let files = FileDialog::new()
                .add_filter("PNG", &["png"])
                .add_filter("JPG", &["jpg"])
                .add_filter("GIF", &["gif"])
                .set_file_name("screenshot")
                .set_directory("/")
                .save_file();
            let ext = files.clone();

            if let Some(mut _img) = self.image.clone() {
                let img = self.crop_image(ctx);
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
        let outline = Rect::from_min_size(self.button_position, self.dimensions);

        let mut resize_frame = outline.clone().shrink2(Vec2::new(5.0, 5.0));
        resize_frame.set_center(outline.center());
        let dx = Rect::from_two_pos(resize_frame.right_bottom(), outline.right_top());
        let top = Rect::from_two_pos(resize_frame.right_top(), outline.left_top());
        let down = Rect::from_two_pos(resize_frame.right_bottom(), outline.left_bottom());
        let sx = Rect::from_two_pos(resize_frame.left_top(), outline.left_bottom());

        ui.painter()
            .rect_stroke(outline, 0.0, Stroke::new(1.0, Color32::RED));
        if response.hovered()
            && ui.rect_contains_pointer(outline)
            && !ui.rect_contains_pointer(resize_frame)
            && !response.dragged()
        {
            if ui.rect_contains_pointer(down) {
                ui.output_mut(|o| {
                    o.cursor_icon = egui::CursorIcon::ResizeNorth;
                });
            } else if ui.rect_contains_pointer(top) {
                ui.output_mut(|o| {
                    o.cursor_icon = egui::CursorIcon::ResizeNorth;
                });
            } else if ui.rect_contains_pointer(dx) {
                ui.output_mut(|o| {
                    o.cursor_icon = egui::CursorIcon::ResizeEast;
                });
            } else if ui.rect_contains_pointer(sx) {
                ui.output_mut(|o| {
                    o.cursor_icon = egui::CursorIcon::ResizeEast;
                });
            }
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
            } else {
                self.frame = TouchedFrame::None;
            }
            self.resizing = true;
        }
        let new_y = self.button_position.y + response.drag_delta().y;
        let new_x = self.button_position.x + response.drag_delta().x;

        match self.frame {
            TouchedFrame::Top => {
                if response.drag_delta().y < 0.0 && new_y > self.display_rect.left_top().y {
                    self.button_position.y += response.drag_delta().y;
                    self.dimensions.y -= response.drag_delta().y;
                } else if response.drag_delta().y >= 0.0
                    && self.dimensions.y - response.drag_delta().y > 15.0
                {
                    self.dimensions.y -= response.drag_delta().y;
                    self.button_position.y += response.drag_delta().y;
                }
            }
            TouchedFrame::Bottom => {
                if response.drag_delta().y < 0.0
                    && self.dimensions.y + response.drag_delta().y > 15.0
                {
                    self.dimensions.y += response.drag_delta().y;
                } else if response.drag_delta().y >= 0.0
                    && new_y + self.dimensions.y < self.display_rect.right_bottom().y
                {
                    self.dimensions.y += response.drag_delta().y;
                }
            }
            TouchedFrame::Right => {
                if response.drag_delta().x < 0.0
                    && self.dimensions.x + response.drag_delta().x > 15.0
                {
                    self.dimensions.x += response.drag_delta().x;
                } else if response.drag_delta().x >= 0.0
                    && new_x + self.dimensions.x < self.display_rect.right_top().x
                {
                    self.dimensions.x += response.drag_delta().x;
                }
            }
            TouchedFrame::Left => {
                if response.drag_delta().x < 0.0 && new_x > self.display_rect.left_bottom().x {
                    self.button_position.x += response.drag_delta().x;
                    self.dimensions.x -= response.drag_delta().x;
                } else if response.drag_delta().x >= 0.0
                    && self.dimensions.x - response.drag_delta().x > 15.0
                {
                    self.button_position.x += response.drag_delta().x;
                    self.dimensions.x -= response.drag_delta().x;
                }
            }
            _ => {}
        }
        if response.drag_released() {
            self.resizing = false;
        }
    }
}
