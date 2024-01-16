use env_logger::fmt::Color;
use screenshots::Screen;
use std::{
    ops::{Add, Sub, RangeInclusive},
    thread::current,
};
use winit::window::CursorIcon;

use eframe::{
    egui::{
        self, pos2, scroll_area::ScrollBarVisibility, Button, CentralPanel, Frame, Id, Layout,
        Pos2, Rect, Sense, Ui, Vec2, ViewportCommand,
    },
    emath::{Align, Align2},
    epaint::{stats::AllocInfo, vec2, Color32, RectShape, Rgba, Rounding, Stroke},
    glow::MINOR_VERSION,
    App,
};

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

struct MyApp {
    state: AppState,
    selected_area: [Pos2; 2],
    crop_area: [Pos2; 2],
    texture: Option<egui::TextureHandle>,
    capture: bool,
    crop: bool,
    image: Option<image::ImageBuffer<image::Rgba<u8>, Vec<u8>>>,
    area: bool,
    //  resizing: bool,
    button_position: Pos2,
    dimensions: Vec2,
    resizing: bool,
    frame: TouchedFrame,
    display_rect: Rect,
    shrink_factor: f32,
    min_pos_top: Pos2,
}
#[derive(Debug)]
enum TouchedFrame {
    None,
    Bottom,
    Top,
    Right,
    Left,
}
impl Default for MyApp {
    fn default() -> Self {
        Self {
            state: AppState::MainApp,
            button_position: Pos2::new(300., 300.),
            dimensions: Vec2::new(100.0, 100.0),
            resizing: false,
            frame: TouchedFrame::None,
            selected_area: [Pos2::ZERO, Pos2::ZERO],
            crop_area: [Pos2::ZERO, Pos2::ZERO],
            texture: None,
            capture: false,
            image: None,
            area: false,
            crop: false,
            display_rect: egui::Rect::ZERO,
            shrink_factor: 0.,
            min_pos_top: Pos2::ZERO
        }
    }
}
enum AppState {
    MainApp,
    NewCapture,
    Selection,
    Crop,
}
impl Default for AppState {
    fn default() -> Self {
        AppState::MainApp
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Rgba::TRANSPARENT.to_rgba_unmultiplied()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let monitor_size: Vec2 = ctx.input(|i| i.viewport().monitor_size.unwrap());
        match self.state {
            AppState::MainApp => {
                //Navbar
                egui::TopBottomPanel::top("buttons navbar").show(ctx, |ui| {
                    //Workaroud for borderless fullscreen transparency bug
                    let monitor_size = monitor_size.add(Vec2::new(1.0, 1.0));

                    //Organize buttons in horizontal navbar
                    ui.horizontal(|ui| {
                        if ui.button("New capture").clicked() {
                            // workaroud to borderless fullscreen render bug when transparent
                            ctx.send_viewport_cmd(ViewportCommand::OuterPosition(Pos2::ZERO));
                            ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(monitor_size));
                            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                                egui::WindowLevel::AlwaysOnTop,
                            ));
                            ctx.send_viewport_cmd(ViewportCommand::Focus);
                            self.state = AppState::NewCapture;
                        }
                        if ui.button("Settings").clicked() {}
                    });
                });

                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(texture) = self.texture.as_ref() {
                        egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                            println!("BBBBBBBBBBBBBBB:{:?}", self.selected_area[0]);
                            let uv = egui::Rect::from_two_pos(
                                Pos2::new(
                                    self.selected_area[0].x / monitor_size.x,
                                    self.selected_area[0].y / monitor_size.y,
                                ),
                                Pos2::new(
                                    self.selected_area[1].x / monitor_size.x,
                                    self.selected_area[1].y / monitor_size.y,
                                ),
                            );

                            //selected is part of image to paint
                            let selection = egui::Rect::from_two_pos(
                                self.selected_area[0],
                                self.selected_area[1],
                            );

                            //space is where image is going to be rendered
                            let center = Pos2::new(monitor_size.x / 2.0, monitor_size.y / 2.0);
                            let mut space = egui::Rect::from_center_size(center, selection.size());

                            /*                             //Modify space in order to render image clearly and not stretched
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
                            } */
                            ui.allocate_ui_with_layout(
                                monitor_size.add(vec2(20.0, 20.0)),
                                Layout::centered_and_justified(egui::Direction::TopDown),
                                |ui| {
                                    let mut a =
                                        ui.allocate_exact_size(selection.size(), Sense::click());
                                    ui.painter().image(texture.id(), a.0, uv, Color32::WHITE);
                                },
                            );
                            /*  let mut a =
                                ui.allocate_at_least(space.size(), egui::Sense::click());
                            a.0.set_center()
                            /* ui.painter().image(texture.id(), a.0, uv, Color32::WHITE); */
                            ui.painter().image(texture.id(), a.0, uv, Color32::WHITE); */

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

/*                                             ctx.send_viewport_cmd(ViewportCommand::OuterPosition(
                                                Pos2::ZERO,
                                            ));
                                            ctx.send_viewport_cmd(ViewportCommand::Decorations(
                                                false,
                                            ));
                                            ctx.send_viewport_cmd(ViewportCommand::InnerSize(
                                                monitor_size,
                                            )); */

                                            self.state = AppState::Crop;
                                        };

                                        if ui.button("Save").clicked() {};
                                    });
                                });
                        });
                    }
                });
            }
            AppState::NewCapture => {
                let pointer: egui::PointerState = ctx.input(|i| i.pointer.clone());

                // Very little opacity for this frame, only to show area that its possible to capture
                let semi_transparent_frame =
                    egui::Frame::none().fill(Color32::WHITE.gamma_multiply(0.1));

                egui::CentralPanel::default()
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
                                    // Store full screen selection
                                    self.selected_area[0] = Pos2::ZERO;
                                    self.selected_area[1] =
                                        Pos2::new(monitor_size.x, monitor_size.y);

                                    //Go to Selection state
                                    self.state = AppState::Selection;

                                    // This makes it skip selection of second point in selection state of app
                                    self.capture = true;

                                    //Request repaint in order to wait until window is transparent
                                    ctx.request_repaint();
                                };

                                if ui.button("Area").clicked() {
                                    self.area = true;
                                };
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

                let transparent_frame = egui::Frame::none().fill(Color32::TRANSPARENT);
                egui::CentralPanel::default()
                    .frame(transparent_frame)
                    .show(ctx, |ui| {
                        //Make pointer into crosshair
                        if ui.ui_contains_pointer() {
                            ctx.output_mut(|o| {
                                o.cursor_icon = egui::CursorIcon::Crosshair;
                            })
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
                            println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA{:?}", self.button_position);
                            println!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA{:?}", self.button_position);
                            self.selected_area[0] = Pos2::ZERO + vec2((self.min_pos_top.x- self.button_position.x) /self.shrink_factor , (self.min_pos_top.y- self.button_position.y )/self.shrink_factor);
                            self.state = AppState::MainApp;
                        }
                    });
                });



                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut p1 = pos2(self.selected_area[0].x/monitor_size.x, self.selected_area[0].y/monitor_size.y);
                    let mut p2 = pos2(self.selected_area[1].x / monitor_size.x,self.selected_area[1].y / monitor_size.y);
                    let av = egui::Rect::from_two_pos(p1,p2);
                    egui::TopBottomPanel::top("top slide").show(ctx, |ui|{
                       ui.add( egui::Slider::new(&mut p1.x, RangeInclusive::new(0.0, 1.0)));
                    });
                    egui::ScrollArea::both()
                        .auto_shrink(true)
                        .drag_to_scroll(true)
                        .show(ui, |ui| {
                            if let Some(texture) = self.texture.as_ref() {

                                let uv = egui::Rect::from_two_pos(Pos2::ZERO, pos2(1., 1.));

                                let monitor_rect = egui::Rect::from_two_pos(
                                    Pos2::ZERO,
                                    Pos2::new(monitor_size.x, monitor_size.y),
                                );
                               
                                let avheight = ui.available_rect_before_wrap().height();
                                let avwidth = avheight *  monitor_rect.aspect_ratio();
                               
                                let rect = Rect::from_center_size(ui.available_rect_before_wrap().center(), vec2(avwidth, avheight));

/*                                  if self.crop {                                    
                                    self.shrink_factor=  avwidth / monitor_rect.width();
                                    let selected_area = Rect::from_center_size(Pos2::ZERO, self.dimensions);
                                    let new_w  = self.dimensions.x * self.shrink_factor;
                                    let new_h = self.dimensions.x * self.shrink_factor / selected_area.aspect_ratio();
                                    let new_x = self.button_position.x *self.shrink_factor;
                                    let new_y = self.button_position.y * self.shrink_factor;

                                    self.min_pos_top = rect.left_top();
                                    self.button_position = rect.left_top()+ vec2(new_x,new_y);
                                    self.dimensions = vec2(new_w,new_h);
                                    self.display_rect = rect;
                                    self.crop = false;
                                }  */                               
                                ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
                                ui.painter().rect_stroke(av, 0.0, Stroke::new(1.0, Color32::RED));
                            }
                        }); 
                });
                 
                /* CentralPanel::default()
                    .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
                    .show(ctx, |ui| {
                        // Draw the button element
                        let pos = self.button_position.clone();
                        let dimensions = self.dimensions.clone();
                        println!("{:?}", pos);
                        MyApp::drag(self, ui, ui.id(), |ui| {
                            let rect = Rect::from_min_size(pos, dimensions);
                            ui.put(
                                rect,
                                Button::new("TEXT")
                                    .fill(Color32::TRANSPARENT)
                                    .sense(Sense::click()),
                            );
                        });
                    }); */
            } 
        }
    }
}

impl MyApp {
    pub fn drag(&mut self, ui: &mut Ui, id: Id, body: impl FnOnce(&mut Ui)) {
        let response = ui.scope(body).response;
        let response = ui.interact(response.rect, id, Sense::drag());
        println!("{:?}", response.rect);
        let mut min = response.rect.right_bottom();
        min.x -= self.dimensions.x;
        min.y -= self.dimensions.y;

        let outline = Rect::from_min_size(min, self.dimensions);

        let mut resize_frame = outline.shrink2(Vec2::new(16.0, 16.0));
        resize_frame.set_center(outline.center());
        let dx = Rect::from_two_pos(resize_frame.right_bottom(), outline.right_top());
        let top = Rect::from_two_pos(resize_frame.right_top(), outline.left_top());
        let down = Rect::from_two_pos(resize_frame.right_bottom(), outline.left_bottom());
        let sx = Rect::from_two_pos(resize_frame.left_top(), outline.left_bottom());

        ui.painter()
            .rect_stroke(outline, 0.0, Stroke::new(3.0, Color32::RED));
        ui.painter()
        .rect_stroke(resize_frame, 0.0, Stroke::new(3.0, Color32::WHITE));
        ui.painter()
            .rect_filled(dx, 0.0, Color32::BLACK);
        ui.painter()
            .rect_stroke(top, 0.0, Stroke::new(3.0, Color32::TRANSPARENT));
        ui.painter()
            .rect_filled(down, 0.0, Color32::BLUE);
        ui.painter()
            .rect_stroke(sx, 0.0, Stroke::new(3.0, Color32::TRANSPARENT));

        if response.hovered()
            && ui.rect_contains_pointer(outline)
            && !ui.rect_contains_pointer(resize_frame)
        {
            println!("outline hovered");
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
        }

         if response.dragged() && !self.resizing {
            println!("dragged not resizing");
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);

            let new_button_position = self.button_position + response.drag_delta();
            let bottom_left_corner = new_button_position + egui::vec2(0., self.dimensions.y);
            let bottom_right_corner = new_button_position + self.dimensions;
            let top_right_coner = new_button_position + egui::vec2(self.dimensions.x, 0.);
            let d = egui::Rect::from_min_size(new_button_position, self.dimensions);
            if self.display_rect.contains_rect(d) {
                self.button_position = new_button_position;
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
            }
            println!("resizing true");
            self.resizing = true;
            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
        }
        if self.resizing && response.dragged() {
            println!("resizing and dragged");
            println!("drag y: {:?}", response.drag_delta().y);
            println!("drag x: {:?}", response.drag_delta().x);
            println!("{:?}", self.frame);
            println!(
                "outline {:?}, dimensions {:?}, button_position {:?}",
                outline, self.dimensions, self.button_position
            );
            println!("rt {:?}", self.display_rect.right_top());
            match self.frame {
                TouchedFrame::Top => {
                    if !(self.dimensions.y - response.drag_delta().y < 50.0
                        || self.button_position.y + response.drag_delta().y
                            < self.display_rect.left_top().y)
                    {
                        self.button_position.y += response.drag_delta().y;
                        self.dimensions.y -= response.drag_delta().y;
                    }
                }
                TouchedFrame::Bottom => {
                    if !(self.dimensions.y + response.drag_delta().y < 50.0
                        || self.button_position.y + self.dimensions.y + response.drag_delta().y
                            > self.display_rect.right_bottom().y)
                    {
                        self.dimensions.y += response.drag_delta().y;
                    }
                }
                TouchedFrame::Right => {
                    if !(self.dimensions.x + response.drag_delta().x < 50.0
                        || self.button_position.x + self.dimensions.x + response.drag_delta().x
                            > self.display_rect.right_top().x)
                    {
                        self.dimensions.x += response.drag_delta().x;
                    }
                }
                TouchedFrame::Left => {
                    if !(self.dimensions.x - response.drag_delta().x < 50.0
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
            println!("released");
            self.resizing = false;
        }
    }
}
