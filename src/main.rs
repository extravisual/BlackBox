use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_inner_size([400.0, 100.0])
            .with_min_inner_size([100.0, 100.0])
            .with_window_level(egui::WindowLevel::AlwaysOnTop)
            .with_transparent(true),
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "Box Overlay",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(PartialEq, Eq)]
enum CursorState {
    Visible,
    AutoHide { last_moved: Option<Instant> },
}

struct App {
    fill: egui::Color32,
    drag_origin: Option<egui::Pos2>,
    alpha: f32,
    virtual_opacity_offset: f32,
    cursor_state: CursorState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            fill: Color32::BLACK.linear_multiply(0.5),
            drag_origin: None,
            alpha: 1.0,
            virtual_opacity_offset: 0.0,
            cursor_state: CursorState::Visible,
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.custom_window_frame(ctx);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}

impl App {
    fn custom_window_frame(&mut self, ctx: &egui::Context) {
        use egui::*;

        let panel_frame = egui::Frame {
            fill: self.fill,
            stroke: Stroke::NONE,
            outer_margin: 0.0.into(),
            ..Default::default()
        };

        CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            let app_rect = ui.max_rect();

            self.title_bar_ui(ui, app_rect);
        });
    }

    fn title_bar_ui(&mut self, ui: &mut egui::Ui, title_bar_rect: eframe::epaint::Rect) {
        use egui::*;

        let opacity_drag_range = 100.0;

        let repaint = ui.input(|i| {
            let mut repaint = false;
            if let CursorState::AutoHide { last_moved } = &mut self.cursor_state {
                if i.pointer.is_moving() {
                    last_moved.replace(Instant::now());
                }

                if last_moved.is_some() {
                    repaint = true;
                }

                if last_moved.is_some_and(|t| t.elapsed() > Duration::from_millis(200)) {
                    last_moved.take();
                }
            }

            if i.pointer.button_clicked(PointerButton::Middle) {
                self.cursor_state = match self.cursor_state {
                    CursorState::Visible => CursorState::AutoHide { last_moved: None },
                    CursorState::AutoHide { .. } => CursorState::Visible,
                }
            }

            if i.pointer.button_released(PointerButton::Secondary) {
                self.drag_origin = None;
            }

            if i.pointer.button_down(PointerButton::Secondary) {
                if self.drag_origin.is_none() {
                    self.drag_origin = i.pointer.interact_pos();
                    self.virtual_opacity_offset =
                        map(1.0 - self.alpha, 0.5, 1.0, 0.0, opacity_drag_range);
                }

                // let press_origin = i.pointer.press_origin();
                self.alpha = self
                    .drag_origin
                    .zip(i.pointer.interact_pos())
                    .map(|(p0, p1)| {
                        map_clamped(
                            -self.virtual_opacity_offset + p0.y - p1.y,
                            0.0,
                            opacity_drag_range,
                            0.5,
                            1.0,
                        )
                    })
                    .unwrap_or(self.alpha);
                // .max(0.5);

                self.fill = Color32::BLACK.gamma_multiply(self.alpha);
            }
            repaint
        });

        if repaint {
            ui.ctx().request_repaint();
        }

        if let Some(drag_origin) = self.drag_origin {
            egui::Area::new("Opacity".into())
                .current_pos(drag_origin - self.virtual_opacity_offset * Vec2::Y)
                .pivot(Align2::CENTER_BOTTOM)
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    ui.add_sized(
                        vec2(12.0, opacity_drag_range),
                        Slider::new(&mut self.alpha, 0.5..=1.0)
                            .vertical()
                            .show_value(false),
                    );
                });
            return;
        }

        let (west, east) = title_bar_rect.split_left_right_at_fraction(0.2);
        let (middle, east) = east.split_left_right_at_fraction(0.75);

        let (northwest, southwest) = west.split_top_bottom_at_fraction(0.2);
        let (west, southwest) = southwest.split_top_bottom_at_fraction(0.75);

        let (northeast, southeast) = east.split_top_bottom_at_fraction(0.2);
        let (east, southeast) = southeast.split_top_bottom_at_fraction(0.75);

        let (north, center) = middle.split_top_bottom_at_fraction(0.2);
        let (center, south) = center.split_top_bottom_at_fraction(0.75);

        let north_west = ui.interact(northwest, "northwest".into(), Sense::click_and_drag());
        let north_east = ui.interact(northeast, "northeast".into(), Sense::click_and_drag());
        let south_west = ui.interact(southwest, "southwest".into(), Sense::click_and_drag());
        let south_east = ui.interact(southeast, "southeast".into(), Sense::click_and_drag());

        let west = ui.interact(west, "west".into(), Sense::click_and_drag());
        let east = ui.interact(east, "east".into(), Sense::click_and_drag());
        let north = ui.interact(north, "north".into(), Sense::click_and_drag());
        let south = ui.interact(south, "south".into(), Sense::click_and_drag());

        let center = ui.interact(center, "center".into(), Sense::click_and_drag());

        let ctx = ui.ctx();

        drag(ctx, &north, ResizeDirection::North);
        drag(ctx, &south, ResizeDirection::South);
        drag(ctx, &east, ResizeDirection::East);
        drag(ctx, &west, ResizeDirection::West);

        drag(ctx, &north_east, ResizeDirection::NorthEast);
        drag(ctx, &north_west, ResizeDirection::NorthWest);
        drag(ctx, &south_west, ResizeDirection::SouthWest);
        drag(ctx, &south_east, ResizeDirection::SouthEast);

        if center.drag_started_by(PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        let show_cursor = match self.cursor_state {
            CursorState::Visible => true,
            CursorState::AutoHide { last_moved } => last_moved.is_some(),
        };

        if show_cursor {
            north.on_hover_cursor(CursorIcon::ResizeNorth);
            south.on_hover_cursor(CursorIcon::ResizeSouth);
            east.on_hover_cursor(CursorIcon::ResizeEast);
            west.on_hover_cursor(CursorIcon::ResizeWest);

            north_east.on_hover_cursor(CursorIcon::ResizeNorthEast);
            north_west.on_hover_cursor(CursorIcon::ResizeNorthWest);
            south_east.on_hover_cursor(CursorIcon::ResizeSouthEast);
            south_west.on_hover_cursor(CursorIcon::ResizeSouthWest);

            center.on_hover_cursor(CursorIcon::Grabbing);
        } else {
            north.on_hover_cursor(CursorIcon::None);
            south.on_hover_cursor(CursorIcon::None);
            east.on_hover_cursor(CursorIcon::None);
            west.on_hover_cursor(CursorIcon::None);

            north_east.on_hover_cursor(CursorIcon::None);
            north_west.on_hover_cursor(CursorIcon::None);
            south_east.on_hover_cursor(CursorIcon::None);
            south_west.on_hover_cursor(CursorIcon::None);

            center.on_hover_cursor(CursorIcon::None);
        }

        if ctx.input(|i| i.key_pressed(Key::Escape) || i.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn drag(ctx: &egui::Context, response: &egui::Response, direction: egui::ResizeDirection) {
    if response.drag_started_by(egui::PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(direction));
    }
}

fn map<X, Y>(x: X, x_min: X, x_max: X, y_min: Y, y_max: Y) -> f32
where
    X: Into<f32>,
    Y: Into<f32>,
{
    let x = x.into();
    let x_min = x_min.into();
    let x_max = x_max.into();
    let y_min = y_min.into();
    let y_max = y_max.into();

    ((x - x_min) * (y_max - y_min) / (x_max - x_min) + y_min).into()
}

fn map_clamped<X: Into<f32>, Y: Into<f32>>(x: X, x_min: X, x_max: X, y_min: Y, y_max: Y) -> f32 {
    let x = x.into();
    let x_min = x_min.into();
    let x_max = x_max.into();
    let y_min = y_min.into();
    let y_max = y_max.into();

    map(x, x_min, x_max, y_min, y_max).clamp(y_min, y_max)
}
