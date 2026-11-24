use craft_ui::{
    docking::{DockArea, DockContent, DockStyle, Layout, Node},
    layout::{SplitAxis, SplitSize},
};
use eframe::egui;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
enum Panel {
    Trace,
    Plot,
}

struct Panels;

impl DockContent<Panel> for Panels {
    fn body(&mut self, ui: &mut egui::Ui, panel: &Panel) {
        match panel {
            Panel::Trace => {
                egui::Grid::new("trace").show(ui, |ui| {
                    ui.strong("Time");
                    ui.strong("Channel");
                    ui.strong("ID");
                    ui.strong("Data");
                    ui.end_row();
                    for (time, channel, id, data) in [
                        ("0.000", "CAN 1", "123", "12 34"),
                        ("0.010", "CAN 2", "1ABCDE", "A5 A5 A5 A5"),
                        ("0.025", "CAN 2", "000", ""),
                    ] {
                        ui.label(time);
                        ui.label(channel);
                        ui.monospace(id);
                        ui.monospace(data);
                        ui.end_row();
                    }
                });
            }
            Panel::Plot => {
                let size = egui::vec2(ui.available_width().max(1.0), 120.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let points = [0.1, 0.2, 0.65, 0.38, 0.85, 0.45]
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        egui::pos2(
                            rect.left() + rect.width() * i as f32 / 5.0,
                            rect.bottom() - rect.height() * value,
                        )
                    })
                    .collect();
                let stroke = egui::Stroke::new(2.0, ui.visuals().selection.stroke.color);
                ui.painter().add(egui::Shape::line(points, stroke));
            }
        }
    }

    fn logical_label(&self, panel: &Panel) -> String {
        match panel {
            Panel::Trace => "Trace",
            Panel::Plot => "Plot",
        }
        .into()
    }
}

struct App {
    layout: Layout<Panel>,
    saved_layout: Option<String>,
    error: Option<String>,
    smoke_frames: Option<u8>,
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.horizontal(|ui| {
            ui.heading("CANexus Studio");
            egui::widgets::global_theme_preference_buttons(ui);
            if ui.button("Save layout").clicked() {
                match serde_json::to_string(&self.layout) {
                    Ok(snapshot) => self.saved_layout = Some(snapshot),
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            if ui
                .add_enabled(
                    self.saved_layout.is_some(),
                    egui::Button::new("Restore layout"),
                )
                .clicked()
            {
                if let Some(snapshot) = &self.saved_layout {
                    match serde_json::from_str(snapshot) {
                        Ok(layout) => self.layout = layout,
                        Err(error) => self.error = Some(error.to_string()),
                    }
                }
            }
        });
        if let Some(error) = &self.error {
            ui.label(error);
        }
        ui.separator();
        let style = DockStyle::from_ui(ui);
        let actions = DockArea::new(egui::Id::unique("analysis-workspace"))
            .show_content(ui, &self.layout, &style, &mut Panels)
            .actions;
        if let Err(error) = self.layout.apply_all(actions, |_| Default::default()) {
            self.error = Some(format!("{error:?}"));
        }
        if let Some(frames) = &mut self.smoke_frames {
            *frames -= 1;
            if *frames == 0 {
                println!("Rendered Trace and Plot dock on the native host");
                self.smoke_frames = None;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ui.ctx().request_repaint();
            }
        }
    }
}

fn fixture_layout() -> Layout<Panel> {
    let tabs = |panel| Node::Tabs {
        panels: vec![panel],
        active: 0,
    };
    Layout {
        root: Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::Ratio(0.65),
            first: Box::new(tabs(Panel::Trace)),
            second: Box::new(tabs(Panel::Plot)),
        }),
        floating: Vec::new(),
    }
}

fn main() -> eframe::Result {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CANexus Studio | Desktop evaluation",
        options,
        Box::new(move |_| {
            Ok(Box::new(App {
                layout: fixture_layout(),
                saved_layout: None,
                error: None,
                smoke_frames: smoke.then_some(3),
            }))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_snapshot_restores_the_same_panels_and_split() {
        let saved = serde_json::to_string(&fixture_layout()).unwrap();
        let restored: Layout<Panel> = serde_json::from_str(&saved).unwrap();
        assert_eq!(serde_json::to_string(&restored).unwrap(), saved);
    }
}
