use std::{cmp::Ordering, fs, io, path::Path};

use chrono::{DateTime, Utc};
use egui::{Key, Label, Modifiers, PointerButton, Sense, TextEdit, Widget};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex};
use egui_extras::{Column, TableBuilder};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct AppData {
    pub favorites: Vec<String>,
    #[serde(skip)]
    pub added_nodes: Vec<(SurfaceIndex, NodeIndex)>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    pub data: AppData,
    #[serde(skip)]
    pub tabs: DockState<Tab>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            data: Default::default(),
            tabs: DockState::new(vec![]),
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
        )]);
        return app;
    }
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.tabs)
                .show_add_buttons(true)
                .style({
                    let mut style = Style::from_egui(ctx.style().as_ref());
                    style.tab_bar.fill_tab_bar = true;
                    style
                })
                .show(ctx, &mut self.data);

            self.data.added_nodes.drain(..).for_each(|(surface, node)| {
                self.tabs.set_focused_node_and_surface((surface, node));
                self.tabs.push_to_focused_leaf(Tab::new2(
                    self.data.favorites.first().unwrap_or(&"/".to_string()),
                ));
            });
        });
    }
}

impl egui_dock::TabViewer for AppData {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match &tab.info {
            Ok(info) => info.file_name.clone().into(),
            Err(_) => "error".into(),
        }
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node));
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        egui::SidePanel::left("favorites").show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.heading("favorites");
                let mut to_remove = None;
                for (i,favorite) in self.favorites.iter().enumerate() {
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
                        tab.new(favorite);
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
                    self.favorites.remove(to_remove);
                }
            });
        });
        if ui.input(|i| i.pointer.button_clicked(PointerButton::Extra1)) {
            // previous
            if !tab.previous_paths.is_empty() {
                let last = tab.previous_paths.remove(tab.previous_paths.len() - 1);
                let mut new = Tab::new2(last.clone());
                new.previous_paths.append(&mut tab.previous_paths);
                new.previous_paths2.append(&mut tab.previous_paths2);
                new.previous_paths2.push(last);
                *tab = new;
            }
        }
        if ui.input(|i| i.pointer.button_clicked(PointerButton::Extra2)) {}
        ui.horizontal(|ui| {
            if ui.button("★").clicked() {
                if !self.favorites.contains(&tab.path) {
                    self.favorites.push(tab.path.clone());
                }
            }
            if ui.button("←").clicked() {}
            if ui.button("→").clicked() {}
            if ui.button("⬆").clicked() {
                let p = tab.path.clone();
                let path = Path::new(&p);
                if let Some(parent) = path.parent() {
                    tab.new(parent.to_str().unwrap_or_default());
                }
            }
            if ui.button("⟳").clicked() {
                tab.new(tab.path.clone());
            }
            let resp = TextEdit::singleline(&mut tab.path)
                .desired_width(ui.available_width())
                .return_key(Some(egui::KeyboardShortcut::new(
                    Modifiers::NONE,
                    Key::Enter,
                )))
                .show(ui);

            if resp.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                tab.new(tab.path.clone());
            }
        });

        if let Ok(entries) = &tab.entries {
            let mut new_path = None;

            TableBuilder::new(ui)
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::auto())
                .sense(egui::Sense::click())
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Date modified");
                    });
                    header.col(|ui| {
                        ui.strong("Size");
                    });
                })
                .body(|mut body| {
                    for entry in entries.iter() {
                        body.row(18.0, |mut row| {
                            row.col(|ui| {
                                //ui.label(entry.file_name.to_string());
                                Label::new(entry.file_name.to_string())
                                    .selectable(false)
                                    .ui(ui);
                            });
                            row.col(|ui| {
                                ui.label(entry.modified.format("%d/%m/%Y %H:%M").to_string());
                            });
                            row.col(|ui| {
                                if entry.file_type.is_file() {
                                    ui.label(bytes_to_human_readable(entry.len));
                                }
                            });

                            let resp = row.response();
                            if resp.double_clicked() && entry.file_type.is_dir() {
                                new_path = Some(entry.path.clone());
                            }

                            resp.context_menu(|ui| {
                                if ui.button("vscode").clicked() {
                                    std::process::Command::new("code")
                                        .arg(entry.path.clone())
                                        .status();
                                    ui.close_menu();
                                }
                                if ui.button("terminal").clicked() {
                                    std::process::Command::new("open")
                                        .arg("-a")
                                        .arg("Terminal")
                                        .arg(entry.path.clone())
                                        .status();
                                    ui.close_menu();
                                }
                                if ui.button("zed").clicked() {
                                    std::process::Command::new("zed")
                                        .arg(entry.path.clone())
                                        .status();
                                    ui.close_menu();
                                }
                            });
                        });
                    }
                });

            if let Some(new_path) = new_path {
                tab.new(new_path);
            }
        }
    }
}

pub struct Tab {
    pub path: String,
    pub info: io::Result<FileEntry>,
    pub entries: Result<Vec<FileEntry>, io::Error>,
    pub previous_paths: Vec<String>,
    pub previous_paths2: Vec<String>,
}

impl Tab {
    fn new2(path: impl Into<String>) -> Self {
        let path = path.into();
        let info = get_meta(&path);
        let entries = get_entries(&path);
        return Self {
            path,
            info,
            entries,
            previous_paths: vec![],
            previous_paths2: vec![],
        };
    }

    fn new(&mut self, path: impl Into<String>) {
        let path = path.into();
        if let Ok(i) = &self.info {
            if i.path == path {
                return;
            }
        }

        let mut new = Self::new2(path);
        new.previous_paths.append(&mut self.previous_paths);
        new.previous_paths.push(self.path.clone());
        *self = new;
    }
}

fn get_meta(path: &str) -> io::Result<FileEntry> {
    let p = Path::new(path);
    let meta = fs::metadata(path)?;
    let file_type = meta.file_type();
    let created: DateTime<Utc> = meta.created()?.into();
    let modified: DateTime<Utc> = meta.modified()?.into();
    let accessed: DateTime<Utc> = meta.accessed()?.into();
    let len = meta.len();

    return Ok(FileEntry {
        len,
        file_type,
        created,
        modified,
        accessed,
        path: path.to_string(),
        file_name: p
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string(),
        selected: false,
    });
}

fn get_entries(path: &str) -> io::Result<Vec<FileEntry>> {
    let mut files = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path().to_str().unwrap_or_default().to_string();
        let file_name = entry.file_name().into_string().unwrap_or_default();
        let meta = entry.metadata()?;
        let file_type = meta.file_type();
        let created: DateTime<Utc> = meta.created()?.into();
        let modified: DateTime<Utc> = meta.modified()?.into();
        let accessed: DateTime<Utc> = meta.accessed()?.into();
        let len = meta.len();

        files.push(FileEntry {
            len,
            file_type,
            created,
            modified,
            accessed,
            path,
            file_name,
            selected: false,
        });
    }
    files.sort_by(|a, b| {
        let type_ord = a.file_type.is_file().cmp(&b.file_type.is_file());
        if type_ord == Ordering::Equal {
            return a.file_name.cmp(&b.file_name);
        }
        return type_ord;
    });
    Ok(files)
}

fn bytes_to_human_readable(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.0} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.0} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.0} B", bytes)
    }
}

pub struct FileEntry {
    pub len: u64,
    pub file_type: fs::FileType,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub path: String,
    pub file_name: String,
    pub selected: bool,
}
