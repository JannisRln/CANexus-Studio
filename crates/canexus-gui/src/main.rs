//! DEV-003: a deliberately small Slint/pbSlintDock integration spike.
//!
//! The dock manager owns layout and panels; this binary owns only presentation.
//! Protocol and data sessions must never be placed inside dockable UI instances.

use std::{cell::RefCell, rc::Rc};

use pbslintdock::{DockManager, DropZone, LayoutOutput, PanelDef, MAIN_WINDOW};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

struct ViewModels {
    panels: Rc<VecModel<PanelVm>>,
    tabs: Rc<VecModel<TabVm>>,
    groups: Rc<VecModel<GroupVm>>,
    splitters: Rc<VecModel<SplitterVm>>,
}

impl ViewModels {
    fn new(window: &MainWindow) -> Self {
        let models = Self {
            panels: Rc::new(VecModel::default()),
            tabs: Rc::new(VecModel::default()),
            groups: Rc::new(VecModel::default()),
            splitters: Rc::new(VecModel::default()),
        };
        let dock = window.global::<PbDock>();
        dock.set_panels(ModelRc::from(models.panels.clone()));
        dock.set_tabs(ModelRc::from(models.tabs.clone()));
        dock.set_groups(ModelRc::from(models.groups.clone()));
        dock.set_splitters(ModelRc::from(models.splitters.clone()));
        models
    }

    fn refresh(&self, window: &MainWindow, manager: &DockManager) {
        let layout = manager.output(MAIN_WINDOW);
        update(&self.panels, layout.panels.iter().map(|p| PanelVm {
            x: p.rect.x, y: p.rect.y, w: p.rect.w, h: p.rect.h, visible: p.visible,
        }).collect());
        update(&self.tabs, layout.tabs.iter().map(|t| TabVm {
            x: t.rect.x, y: t.rect.y, w: t.rect.w, h: t.rect.h,
            title: SharedString::from(&t.title), active: t.active, closable: t.closable,
        }).collect());
        update(&self.groups, layout.groups.iter().map(|g| GroupVm {
            x: g.rect.x, y: g.rect.y, w: g.rect.w, h: g.rect.h, bar_h: g.tabbar.h,
        }).collect());
        update(&self.splitters, layout.splitters.iter().map(|s| SplitterVm {
            x: s.rect.x, y: s.rect.y, w: s.rect.w, h: s.rect.h, vertical: s.vertical,
        }).collect());
        set_overlay(window, manager, &layout);
    }
}

fn set_overlay(window: &MainWindow, manager: &DockManager, _layout: &LayoutOutput) {
    let overlay = manager.overlay();
    let state = window.global::<PbDock>();
    state.set_overlay_active(overlay.active && overlay.window == MAIN_WINDOW);
    state.set_overlay_zone(overlay.zone as i32);
    state.set_ov_x(overlay.preview.x);
    state.set_ov_y(overlay.preview.y);
    state.set_ov_w(overlay.preview.w);
    state.set_ov_h(overlay.preview.h);
    state.set_gr_x(overlay.group_rect.x);
    state.set_gr_y(overlay.group_rect.y);
    state.set_gr_w(overlay.group_rect.w);
    state.set_gr_h(overlay.group_rect.h);
    state.set_ghost_active(false);
}

fn update<T: Clone + 'static>(model: &Rc<VecModel<T>>, rows: Vec<T>) {
    let common = model.row_count().min(rows.len());
    for (index, value) in rows.iter().take(common).enumerate() {
        model.set_row_data(index, value.clone());
    }
    while model.row_count() > rows.len() {
        model.remove(model.row_count() - 1);
    }
    for value in rows.into_iter().skip(common) {
        model.push(value);
    }
}

fn initial_dock() -> DockManager {
    let dock = DockManager::new();
    for (id, label) in [
        ("explorer", "Explorer"),
        ("trace", "Trace"),
        ("details", "Details"),
    ] {
        dock.add_panel(PanelDef::new(id, label));
    }
    dock.dock_panel_root("trace", DropZone::RootLeft, 0.7);
    dock.dock_panel("explorer", "trace", DropZone::Left, 0.25);
    dock.dock_panel("details", "trace", DropZone::Bottom, 0.3);
    dock
}

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;
    let manager = Rc::new(initial_dock());
    let models = Rc::new(ViewModels::new(&window));
    let layout_snapshot = Rc::new(RefCell::new(None::<String>));

    let sync: Rc<dyn Fn()> = {
        let weak = window.as_weak();
        let manager = manager.clone();
        let models = models.clone();
        Rc::new(move || {
            if let Some(window) = weak.upgrade() {
                models.refresh(&window, &manager);
            }
        })
    };
    manager.set_changed_handler({
        let weak = Rc::downgrade(&sync);
        move || {
            if let Some(sync) = weak.upgrade() {
                sync();
            }
        }
    });

    let dock = window.global::<PbDock>();
    dock.on_slot_of({
        let manager = manager.clone();
        move |id| manager.slot_of(id.as_str())
    });
    dock.on_view_resized({
        let manager = manager.clone();
        move |w, h| manager.set_view_size(MAIN_WINDOW, w, h)
    });
    dock.on_tab_pressed({
        let manager = manager.clone();
        let weak = window.as_weak();
        move |index, x, y| {
            if let Some(window) = weak.upgrade() {
                update_origin(&window, &manager, x, y);
            }
            manager.tab_pressed(MAIN_WINDOW, index, x, y);
        }
    });
    dock.on_group_bar_pressed({
        let manager = manager.clone();
        move |index, x, y| manager.group_bar_pressed(MAIN_WINDOW, index, x, y)
    });
    dock.on_tab_close({
        let manager = manager.clone();
        move |index| manager.tab_close(MAIN_WINDOW, index)
    });
    dock.on_pointer_moved({
        let manager = manager.clone();
        move |x, y| manager.pointer_moved(MAIN_WINDOW, x, y)
    });
    dock.on_pointer_released({
        let manager = manager.clone();
        move |x, y| manager.pointer_released(MAIN_WINDOW, x, y)
    });
    dock.on_splitter_pressed({
        let manager = manager.clone();
        move |index, point| manager.splitter_pressed(MAIN_WINDOW, index, point)
    });
    dock.on_splitter_dragged({
        let manager = manager.clone();
        move |index, point| manager.splitter_dragged(MAIN_WINDOW, index, point)
    });
    window.on_toggle_panel({
        let manager = manager.clone();
        move |id| {
            if manager.is_open(id.as_str()) {
                manager.close_panel(id.as_str());
            } else {
                manager.open_panel(id.as_str());
            }
        }
    });
    window.on_save_layout({
        let manager = manager.clone();
        let snapshot = layout_snapshot.clone();
        move || *snapshot.borrow_mut() = Some(manager.save_layout())
    });
    window.on_load_layout({
        let manager = manager.clone();
        move || {
            if let Some(layout) = layout_snapshot.borrow().as_ref() {
                let _ = manager.restore_layout(layout);
            }
        }
    });
    sync();
    window.run()
}

fn update_origin(window: &MainWindow, dock: &DockManager, x: f32, y: f32) {
    if let Some((cx, cy)) = DockManager::cursor_screen_pos() {
        let scale = window.window().scale_factor().max(1.);
        dock.set_window_origin(MAIN_WINDOW, cx / scale - x, cy / scale - y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_survives_close_and_restore() {
        let manager = initial_dock();
        manager.set_view_size(MAIN_WINDOW, 1280., 800.);
        assert_eq!(manager.panel_count(), 3);
        assert_eq!(manager.output(MAIN_WINDOW).groups.len(), 3);
        let layout = manager.save_layout();
        manager.close_panel("explorer");
        assert!(!manager.is_open("explorer"));
        assert!(manager.restore_layout(&layout));
        assert!(manager.is_open("explorer"));
    }

    #[test]
    fn can_float_and_redock_without_losing_panel_registration() {
        let manager = initial_dock();
        manager.float_panel("details", 100., 100., 400., 320.);
        assert_eq!(manager.float_ids().len(), 1);
        manager.dock_panel("details", "trace", DropZone::Right, 0.3);
        assert!(manager.float_ids().is_empty());
        assert_eq!(manager.panel_count(), 3);
    }
}
