use eframe::egui::{
    self, panel::TopBottomSide, pos2, Button, CentralPanel, Frame, Key, Pos2, Rect, Sense,
    TopBottomPanel, ViewportCommand,
};
use eframe::epaint::{vec2, Color32, Rounding, Stroke};
use screenshots::Screen;
use std::ops::{Add, Div};
use std::{thread, time};

use super::AppState;
use super::MyApp;
impl MyApp {
    pub fn main_state_visuals(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("buttons navbar").show(ctx, |ui| {
            //Organize buttons in horizontal navbar
            ui.horizontal(|ui| {
                if ui.button("New capture now").clicked() {
                    self.delay = 0;
                    self.set_new_capture_window(ctx);
                    self.state = AppState::NewCapture;
                }
                ui.add_space(20.0);

                if ui.button("New capture after:").clicked() {
                    self.set_new_capture_window(ctx);
                    if self.delay == 0 {
                        self.state = AppState::NewCapture;
                    } else {
                        ctx.send_viewport_cmd(ViewportCommand::Visible(false));
                        ctx.request_repaint();
                        self.state = AppState::Selection;
                    }
                }
                ui.add(egui::Slider::new(&mut self.delay, 0..=60).text("seconds"));

                ui.add_space(ui.available_size().x - 50.0);
                if ui.button("Settings").clicked() {
                    self.delay = 0;
                    self.state = AppState::Settings;
                }
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = self.texture.clone() {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    //IMAGE RENDERING
                    let uv = self.calculate_uv(ctx);
                    let space = self.calculate_space(ctx, ui);
                    ui.painter().image(texture.id(), space, uv, Color32::WHITE);

                    //OPTIONS
                    egui::Window::new("options")
                        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                        .collapsible(false)
                        .resizable(false)
                        .title_bar(false)
                        .show(ctx, |ui| {
                            ui.vertical(|ui| {
                                if ui.button("Save ").clicked() {
                                    self.save_capture(ctx);
                                }
                                if ui.button("Crop ").clicked() {
                                    self.handle_crop_request(ctx);
                                }
                                if ui.button("Copy ").clicked() {
                                    self.copy_to_clipboard(ctx);
                                }
                            });
                        });
                });
            }
        });
    }
    pub fn newcapture_state_visuals(&mut self, ctx: &egui::Context) {
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
                            self.handle_fullscreen_capture(ctx);
                        }

                        if ui.button("Area").clicked() {
                            self.area = true;
                        }
                    });
                    // Selection if button has been pressed, must do it this way otherwise button click is recorded as first point of selection
                    if pointer.primary_clicked() && !ui.ui_contains_pointer() {
                        self.area = true;
                        self.selected_area[0] = ctx.input(|i| i.pointer.interact_pos().unwrap());
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
    pub fn selection_state_visuals(&mut self, ctx: &egui::Context) {
        //reset option window in type of selection
        self.area = false;

        let transparent_frame = Frame::none().fill(Color32::TRANSPARENT);
        CentralPanel::default()
            .frame(transparent_frame)
            .show(ctx, |ui| {
                if self.delay > 0 {
                    thread::sleep(time::Duration::from_secs(self.delay));
                    self.delay = 0;
                    ctx.send_viewport_cmd(ViewportCommand::Visible(true));
                    self.state = AppState::NewCapture;
                }
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
                    ui.painter()
                        .rect_stroke(rect, Rounding::ZERO, Stroke::new(1.0, Color32::RED));

                    if pointer.primary_released() {
                        if pointer_pos == self.selected_area[0] {
                            self.state = AppState::NewCapture;
                        } else {
                            self.selected_area[1] = pointer_pos;
                            self.capture = true;
                        }
                        ctx.request_repaint();
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
                    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(egui::WindowLevel::Normal));

                    //Change state to Main state
                    self.state = AppState::MainApp;
                }
            });
    }
    pub fn crop_state_visuals(&mut self, ctx: &egui::Context, monitor_rect: Rect) {
        egui::TopBottomPanel::top("buttons navbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("CONFIRM").clicked() {
                    self.button_position.x =
                        (self.button_position.x - self.display_rect.left_top().x) /
                        self.shrink_factor;
                    self.button_position.y =
                        (self.button_position.y - self.display_rect.left_top().y) /
                        self.shrink_factor;
                    self.selected_area[0] = self.button_position;
                    self.dimensions = self.dimensions.div(self.shrink_factor);
                    self.selected_area[1] = self.selected_area[0].add(self.dimensions);
                    self.state = AppState::MainApp;
                }

                ui.add_space(ui.available_size().x / 3.3);
                ui.label(
                    "Click and drag any side to resize the selection, click confirm to save new selection"
                );
                ui.add_space(ui.available_size().x - 50.0);
                if ui.button("CANCEL").clicked() {
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
                        let avheight = ui.available_rect_before_wrap().shrink(60.0).height();
                        let avwidth = avheight * monitor_rect.aspect_ratio();
                        let rect = Rect::from_center_size(
                            ui.available_rect_before_wrap().center(),
                            vec2(avwidth, avheight),
                        );

                        if self.crop {
                            self.shrink_factor = avwidth / monitor_rect.width();
                            let selected_area = Rect::from_center_size(Pos2::ZERO, self.dimensions);
                            let new_w = self.dimensions.x * self.shrink_factor;
                            let new_h = (self.dimensions.x * self.shrink_factor)
                                / selected_area.aspect_ratio();
                            let new_x = self.button_position.x * self.shrink_factor;
                            let new_y = self.button_position.y * self.shrink_factor;

                            self.min_pos_top = rect.left_top();
                            self.button_position = rect.left_top() + vec2(new_x, new_y);
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
                    let rect: Rect = Rect::from_min_size(pos, dimensions);
                    ui.put(
                        rect,
                        Button::new("")
                            .fill(Color32::TRANSPARENT)
                            .sense(Sense::click()),
                    );
                });
            });
    }
    pub fn settings_state_visuals(&mut self, ctx: &egui::Context) {
        TopBottomPanel::new(TopBottomSide::Top, "go back").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Settings");
                ui.add_space(ui.available_size().x - 50.0);
                if ui.button("Go back").clicked() {
                    self.state = AppState::MainApp;
                    ctx.request_repaint()
                }
            });
        });
        CentralPanel::default().show(ctx, |ui| {
            ui.label(
                "Modify keybiding by hovering on key with mouse and pressing new desidered key on keyboard, new choosen key must not be already assigned"
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
                                !self.key_bindings.is_key_assigned(key.to_owned())
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
                                !self.key_bindings.is_key_assigned(key.to_owned())
                            {
                                self.key_bindings.new = key.to_owned();
                            }
                        }
                    })
                }
            });
            ui.horizontal(|ui| {
                ui.label("Save current capture: ");
                ui.add_enabled(false, Button::new("Ctrl"));
                ui.label("+");
                if ui.button(format!("{:?}", self.key_bindings.save)).hovered() {
                    ui.input(|i| {
                        for key in Key::ALL {
                            if
                                i.key_pressed(key.to_owned()) &&
                                !self.key_bindings.is_key_assigned(key.to_owned())
                            {
                                self.key_bindings.save = key.to_owned();
                            }
                        }
                    })
                }
            });
            ui.horizontal(|ui| {
                ui.label("Cancel selection resizing: ");
                ui.add_enabled(false, Button::new("Ctrl"));
                ui.label("+");
                if ui.button(format!("{:?}", self.key_bindings.cancel)).hovered() {
                    let input = ui.input(|i| i.clone());
                    for key in Key::ALL {
                        if
                            input.key_pressed(key.to_owned()) &&
                            !self.key_bindings.is_key_assigned(key.to_owned())
                        {
                            self.key_bindings.cancel = key.to_owned();
                        }
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("Resize selection: ");
                ui.add_enabled(false, Button::new("Ctrl"));
                ui.label("+");
                if ui.button(format!("{:?}", self.key_bindings.crop)).hovered() {
                    ui.input(|i| {
                        for key in Key::ALL {
                            if
                                i.key_pressed(key.to_owned()) &&
                                !self.key_bindings.is_key_assigned(key.to_owned())
                            {
                                self.key_bindings.crop = key.to_owned();
                            }
                        }
                    })
                }
            });

            ui.horizontal(|ui| {
                ui.label("Copy image to clipboard: ");
                ui.add_enabled(false, Button::new("Ctrl"));
                ui.label("+");
                if ui.button(format!("{:?}", self.key_bindings.clipboard)).hovered() {
                    ui.input(|i| {
                        for key in Key::ALL {
                            if
                                i.key_pressed(key.to_owned()) &&
                                !self.key_bindings.is_key_assigned(key.to_owned())
                            {
                                self.key_bindings.clipboard = key.to_owned();
                            }
                        }
                    })
                }
            });
        });
    }
}
