use std::path::Path;

use egui::{Label, Sense, Widget};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};

use crate::{actions::actions, tab::Tab, tabviewer::AppData};



#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    pub data: AppData,
    #[serde(skip)]
    pub tabs: DockState<Tab>,
    pub latest_tab_id: u64,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: Default::default(),
            tabs: DockState::new(vec![]),
            latest_tab_id: 0,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        app.tabs = DockState::new(vec![Tab::new2(
            app.data.favorites.first().unwrap_or(&"/".to_string()),
            egui::Id::new(app.latest_tab_id),
        )]);
        app.latest_tab_id += 1;
        //app.tabs.set_active_tab((SurfaceIndex(0), NodeIndex(0), TabIndex(0)));
        app.tabs
            .set_focused_node_and_surface((SurfaceIndex(0), NodeIndex(0)));
        app.data.actions = actions();
        return app;
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("favorites_tab").show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("favorites");
                let mut to_remove = None;
                for (i, favorite) in self.data.favorites.iter().enumerate() {
                    let path = Path::new(favorite);
                    let file_name = path
                        .file_name()
                        .and_then(|x| Some(x.to_str().unwrap_or_default()))
                        .unwrap_or("unkown");
                    let resp = Label::new(file_name)
                        .sense(Sense::click())
                        .selectable(false)
                        .ui(ui);
                    if resp.clicked() {
                        let tab = self.tabs.find_active_focused();
                        if let Some((rect, tab)) = tab {
                            tab.refresh(favorite);
                        }
                    }
                    resp.context_menu(|ui| {
                        if Label::new("x")
                            .sense(Sense::click())
                            .selectable(false)
                            .ui(ui)
                            .clicked()
                        {
                            ui.close_menu();
                            to_remove = Some(i);
                        }
                    });
                }
                if let Some(to_remove) = to_remove {
                    self.data.favorites.remove(to_remove);
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.tabs)
                .show_add_buttons(true)
                .style({
                    let mut style = Style::from_egui(ctx.style().as_ref());
                    style.tab_bar.fill_tab_bar = true;
                    style
                })
                .show(ctx, &mut self.data);

            self.latest_tab_id += 1;
            self.data.added_nodes.drain(..).for_each(|(surface, node)| {
                self.tabs.set_focused_node_and_surface((surface, node));
                self.tabs.push_to_focused_leaf(Tab::new2(
                    self.data.favorites.first().unwrap_or(&"/".to_string()),
                    egui::Id::new(self.latest_tab_id),
                ));
            });
        });
    }
}

