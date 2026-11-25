mod capture;

use canexus_core::{
    can::{CanFormat, CanFrame},
    event::Observed,
};
use capture::{Capture, Snapshot, ROW_LIMIT};
use craft_ui::{
    docking::{DockArea, DockContent, DockStyle, Layout, Node},
    layout::{SplitAxis, SplitSize},
};
use eframe::egui;
use std::{
    fmt::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Panel {
    Trace,
    Inspector,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Column {
    Sequence,
    Time,
    Channel,
    Id,
    Format,
}

const COLUMNS: [(Column, &str, f32); 5] = [
    (Column::Sequence, "Sequence", 80.0),
    (Column::Time, "Time (ms)", 90.0),
    (Column::Channel, "Channel", 72.0),
    (Column::Id, "CAN ID", 100.0),
    (Column::Format, "Format", 85.0),
];

struct Trace {
    id: String,
    format: Option<CanFormat>,
    sort: Column,
    descending: bool,
    follow: bool,
    selected: Option<Observed<CanFrame>>,
}

impl Default for Trace {
    fn default() -> Self {
        Self {
            id: String::new(),
            format: None,
            sort: Column::Sequence,
            descending: false,
            follow: true,
            selected: None,
        }
    }
}

impl Trace {
    fn rows<'a>(
        &self,
        snapshot: &'a Snapshot,
    ) -> Result<Vec<&'a Observed<CanFrame>>, &'static str> {
        let id_filter = match self.id.trim() {
            "" => None,
            query => {
                let value = query
                    .strip_prefix("0x")
                    .or_else(|| query.strip_prefix("0X"))
                    .unwrap_or(query);
                Some(
                    u32::from_str_radix(value, 16)
                        .ok()
                        .filter(|id| *id <= 0x1fff_ffff)
                        .ok_or("Enter a hexadecimal CAN ID from 0 to 1FFFFFFF.")?,
                )
            }
        };
        let mut rows: Vec<_> = snapshot
            .rows
            .iter()
            .filter(|event| {
                id_filter.is_none_or(|id| event.value.id().raw() == id)
                    && self
                        .format
                        .is_none_or(|format| event.value.flags().format == format)
            })
            .collect();
        rows.sort_unstable_by_key(|event| {
            let key = match self.sort {
                Column::Sequence => event.id.sequence,
                Column::Time => event.timestamp.ticks,
                Column::Channel => u64::from(event.channel.get()),
                Column::Id => u64::from(event.value.id().raw()),
                Column::Format => u64::from(event.value.flags().format == CanFormat::Fd),
            };
            (key, event.id.sequence)
        });
        if self.descending {
            rows.reverse();
        }
        Ok(rows)
    }
}

struct Panels<'a> {
    snapshot: &'a Snapshot,
    trace: &'a mut Trace,
}

impl DockContent<Panel> for Panels<'_> {
    fn body(&mut self, ui: &mut egui::Ui, panel: &Panel) {
        match panel {
            Panel::Trace => self.trace(ui),
            Panel::Inspector => inspector(ui, self.trace.selected.as_ref()),
        }
    }

    fn logical_label(&self, panel: &Panel) -> String {
        match panel {
            Panel::Trace => "Trace",
            Panel::Inspector => "Inspector",
        }
        .into()
    }
}

impl Panels<'_> {
    fn trace(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("CAN ID");
            ui.add(
                egui::TextEdit::singleline(&mut self.trace.id)
                    .hint_text("Hexadecimal")
                    .desired_width(100.0),
            );
            ui.selectable_value(&mut self.trace.format, None, "All");
            ui.selectable_value(&mut self.trace.format, Some(CanFormat::Classic), "Classic");
            ui.selectable_value(&mut self.trace.format, Some(CanFormat::Fd), "FD");
            ui.checkbox(&mut self.trace.follow, "Follow");
        });
        ui.add_space(8.0);
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                for (column, label, width) in COLUMNS {
                    let label = if self.trace.sort == column {
                        format!("{label} {}", if self.trace.descending { "↓" } else { "↑" })
                    } else {
                        label.to_string()
                    };
                    if ui
                        .add_sized([width, 24.0], egui::Button::new(label))
                        .clicked()
                    {
                        self.trace.descending = self.trace.sort == column && !self.trace.descending;
                        self.trace.sort = column;
                    }
                }
                ui.strong("Data");
            });
            ui.separator();
            let rows = match self.trace.rows(self.snapshot) {
                Ok(rows) => rows,
                Err(error) => {
                    ui.label(error);
                    return;
                }
            };
            if rows.is_empty() {
                ui.label("No frames match the current view.");
                return;
            }
            let follow =
                self.trace.follow && self.trace.sort == Column::Sequence && !self.trace.descending;
            egui::ScrollArea::vertical()
                .stick_to_bottom(follow)
                .show_rows(ui, 25.0, rows.len(), |ui, visible| {
                    for event in &rows[visible] {
                        ui.horizontal(|ui| {
                            let selected = self
                                .trace
                                .selected
                                .as_ref()
                                .is_some_and(|row| row.id == event.id);
                            if ui
                                .add_sized(
                                    [80.0, 25.0],
                                    egui::Button::selectable(
                                        selected,
                                        event.id.sequence.to_string(),
                                    ),
                                )
                                .clicked()
                            {
                                self.trace.selected = Some(**event);
                            }
                            ui.add_sized(
                                [90.0, 25.0],
                                egui::Label::new(event.timestamp.ticks.to_string()),
                            );
                            ui.add_sized(
                                [72.0, 25.0],
                                egui::Label::new(format!("CAN {}", event.channel.get() + 1)),
                            );
                            ui.add_sized(
                                [100.0, 25.0],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:X}", event.value.id().raw()))
                                        .monospace(),
                                ),
                            );
                            ui.add_sized(
                                [85.0, 25.0],
                                egui::Label::new(format_name(event.value.flags().format)),
                            );
                            ui.monospace(payload(event.value.data()));
                        });
                    }
                });
        });
    }
}

fn format_name(format: CanFormat) -> &'static str {
    match format {
        CanFormat::Classic => "Classic",
        CanFormat::Fd => "FD",
    }
}

fn payload(bytes: &[u8]) -> String {
    let mut payload = String::with_capacity(bytes.len() * 3);
    for &byte in bytes {
        if !payload.is_empty() {
            payload.push(' ');
        }
        write!(payload, "{byte:02X}").expect("string write");
    }
    payload
}

fn inspector(ui: &mut egui::Ui, selected: Option<&Observed<CanFrame>>) {
    let Some(event) = selected else {
        ui.label("Select a sequence in Trace to inspect the frame.");
        return;
    };
    ui.heading(format!("0x{:X}", event.value.id().raw()));
    ui.label(format!(
        "Sequence {} · {} ms",
        event.id.sequence, event.timestamp.ticks
    ));
    ui.label(format!(
        "CAN {} · {:?} · {}",
        event.channel.get() + 1,
        event.direction,
        format_name(event.value.flags().format)
    ));
    ui.label(if event.value.id().is_extended() {
        "Extended identifier"
    } else {
        "Standard identifier"
    });
    ui.label(format!(
        "DLC {} · {} payload bytes",
        event.value.dlc(),
        event.value.data_len()
    ));
    ui.label(format!(
        "Remote: {} · Bit rate switch: {}",
        event.value.flags().remote,
        event.value.flags().bit_rate_switch
    ));
    for line in event.value.data().chunks(8) {
        ui.monospace(payload(line));
    }
}

struct App {
    layout: Layout<Panel>,
    trace: Trace,
    output: Arc<Mutex<Snapshot>>,
    frozen: Option<Snapshot>,
    capture: Option<Capture>,
    error: Option<String>,
    smoke: bool,
    smoke_stopped: bool,
}

impl App {
    fn new(smoke: bool) -> Self {
        let mut app = Self {
            layout: default_layout(),
            trace: Trace::default(),
            output: Arc::new(Mutex::new(Snapshot::default())),
            frozen: None,
            capture: None,
            error: None,
            smoke,
            smoke_stopped: false,
        };
        if smoke {
            app.start();
        }
        app
    }

    fn start(&mut self) {
        self.output = Arc::new(Mutex::new(Snapshot::default()));
        self.trace.selected = None;
        self.frozen = None;
        self.error = None;
        match Capture::start(Arc::clone(&self.output)) {
            Ok(capture) => self.capture = Some(capture),
            Err(error) => self.error = Some(format!("Unable to start capture: {error}")),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.capture.as_ref().is_some_and(Capture::is_finished) {
            self.capture = None;
        }
        let snapshot = self.output.lock().expect("trace snapshot").clone();
        ui.horizontal(|ui| {
            ui.heading("CANexus Studio");
            ui.separator();
            ui.label("Virtual CAN / CAN FD");
            if ui
                .add_enabled(self.capture.is_none(), egui::Button::new("Start"))
                .clicked()
            {
                self.start();
            }
            if let Some(capture) = &mut self.capture {
                if ui
                    .add_enabled(!capture.is_stopping(), egui::Button::new("Stop"))
                    .clicked()
                {
                    capture.stop();
                }
            }
            if ui
                .button(if self.frozen.is_some() {
                    "Resume view"
                } else {
                    "Freeze view"
                })
                .clicked()
            {
                self.frozen = self.frozen.is_none().then(|| snapshot.clone());
            }
            if ui.button("Reset panels").clicked() {
                self.layout = default_layout();
            }
            egui::widgets::global_theme_preference_buttons(ui);
        });
        ui.horizontal_wrapped(|ui| {
            let state = match &self.capture {
                Some(capture) if capture.is_stopping() => "Stopping",
                Some(_) => "Capturing",
                None => "Stopped",
            };
            ui.strong(state);
            ui.label(format!(
                "Accepted {}   RAM rejected {}   Queue lost {}   Trace evicted {}",
                snapshot.accepted, snapshot.rejected, snapshot.receiver_loss, snapshot.evicted
            ));
            if self.frozen.is_some() {
                ui.label("View frozen");
            }
        });
        if let Some(error) = self.error.as_ref().or(snapshot.error.as_ref()) {
            ui.label(error);
        }
        ui.separator();
        let style = DockStyle::from_ui(ui);
        let actions = DockArea::new(egui::Id::unique("analysis-workspace"))
            .show_content(
                ui,
                &self.layout,
                &style,
                &mut Panels {
                    snapshot: self.frozen.as_ref().unwrap_or(&snapshot),
                    trace: &mut self.trace,
                },
            )
            .actions;
        if let Err(error) = self.layout.apply_all(actions, |_| Default::default()) {
            self.error = Some(format!("{error:?}"));
        }
        if self.capture.is_some() {
            ui.ctx().request_repaint_after(Duration::from_millis(30));
        }
        if self.smoke {
            assert!(self.error.is_none() && snapshot.error.is_none());
            if !self.smoke_stopped
                && snapshot
                    .rows
                    .iter()
                    .any(|row| row.value.flags().format == CanFormat::Fd)
            {
                self.capture.as_mut().expect("capture running").stop();
                self.smoke_stopped = true;
            }
            if self.smoke_stopped && self.capture.is_none() {
                assert!(snapshot
                    .rows
                    .iter()
                    .any(|row| row.value.flags().format == CanFormat::Classic));
                assert!(snapshot.rows.len() <= ROW_LIMIT);
                assert_eq!(snapshot.accepted, snapshot.rows.len() as u64);
                println!(
                    "Opened CANexus, captured Classic and FD, rendered live trace and stopped."
                );
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

fn default_layout() -> Layout<Panel> {
    let tabs = |panel| Node::Tabs {
        panels: vec![panel],
        active: 0,
    };
    Layout {
        root: Some(Node::Split {
            axis: SplitAxis::Horizontal,
            size: SplitSize::Ratio(0.73),
            first: Box::new(tabs(Panel::Trace)),
            second: Box::new(tabs(Panel::Inspector)),
        }),
        floating: Vec::new(),
    }
}

#[cfg(windows)]
fn configure_renderer(options: &mut eframe::NativeOptions) {
    let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle();
    setup.instance_descriptor.backends = wgpu::Backends::DX12;
    options.wgpu_options.wgpu_setup = eframe::egui_wgpu::WgpuSetup::CreateNew(setup);
}

#[cfg(not(windows))]
fn configure_renderer(_: &mut eframe::NativeOptions) {}

fn main() -> eframe::Result {
    let smoke = std::env::args().any(|arg| arg == "--smoke");
    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1260.0, 760.0])
            .with_min_inner_size([740.0, 420.0]),
        ..Default::default()
    };
    configure_renderer(&mut options);
    eframe::run_native(
        "CANexus Studio",
        options,
        Box::new(move |_| Ok(Box::new(App::new(smoke)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use canexus_core::{
        event::RawEventId,
        time::{ClockDomainId, TimestampResolution},
        virtual_can::{fixtures, VirtualCanAdapter},
        SourceId,
    };

    #[test]
    fn trace_filters_and_sorts_captured_rows() {
        let fixture = fixtures::mixed_classic_fd();
        let mut adapter = VirtualCanAdapter::new(
            SourceId::new(1),
            ClockDomainId::new(1),
            TimestampResolution::new(1, 1).unwrap(),
            &fixture,
        )
        .unwrap();
        adapter.start().unwrap();
        adapter.advance_to(25).unwrap();
        let rows = std::iter::from_fn(|| adapter.next_due().unwrap()).collect();
        let snapshot = Snapshot {
            rows,
            ..Snapshot::default()
        };
        let mut trace = Trace {
            id: "0X1ABCDE".into(),
            ..Trace::default()
        };
        assert_eq!(
            trace.rows(&snapshot).unwrap()[0].id,
            RawEventId {
                source: SourceId::new(1),
                sequence: 1
            }
        );
        trace.id = "bogus".into();
        assert!(trace.rows(&snapshot).is_err());
        trace.id.clear();
        trace.sort = Column::Id;
        trace.descending = true;
        trace.format = Some(CanFormat::Classic);
        assert_eq!(
            trace
                .rows(&snapshot)
                .unwrap()
                .iter()
                .map(|row| row.id.sequence)
                .collect::<Vec<_>>(),
            [2, 0, 3]
        );
    }
}
