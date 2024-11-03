use crate::{
    actions::Action,
    files::{bytes_to_human_readable, FileEntry},
    tab::Tab,
};
use std::{borrow::Borrow, path::Path};

use egui::{Key, Label, Modifiers, PointerButton, Sense, TextEdit, Widget};
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabIndex};
use egui_extras::{Column, TableBuilder};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct AppData {
    pub favorites: Vec<String>,
    #[serde(skip)]
    pub added_nodes: Vec<(SurfaceIndex, NodeIndex)>,
    #[serde(skip)]
    pub actions: Vec<Action>,
}

impl egui_dock::TabViewer for AppData {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match &tab.info {
            Ok(info) => info.path.clone().into(),
            Err(_) => "invalid path".into(),
        }
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        tab.id
    }

    fn on_add(&mut self, surface: SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node));
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.state.relead = false;

        if ui.input(|i| i.pointer.button_clicked(PointerButton::Extra1)) {
            // previous
            if !tab.previous_paths.is_empty() {
                let last = tab.previous_paths.remove(tab.previous_paths.len() - 1);
                let mut new = Tab::new2(last.clone(), tab.id);
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
                    tab.refresh(parent.to_str().unwrap_or_default());
                }
            }
            if ui.button("⟳").clicked() {
                tab.refresh_hard(tab.path.clone());
            }
            let search_width = 150.0;
            let resp = TextEdit::singleline(&mut tab.path)
                .desired_width(ui.available_width() - search_width)
                .return_key(Some(egui::KeyboardShortcut::new(
                    Modifiers::NONE,
                    Key::Enter,
                )))
                .cursor_at_end(true)
                .show(ui);

            if resp.response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                tab.refresh(tab.path.clone());
            }

            TextEdit::singleline(&mut tab.search)
                .return_key(Some(egui::KeyboardShortcut::new(
                    Modifiers::NONE,
                    Key::Enter,
                )))
                .hint_text("search")
                .cursor_at_end(true)
                .desired_width(search_width - 50.)
                .show(ui);

            if !tab.search.is_empty() {
                if ui.button("X").clicked() {
                    tab.search = "".into();
                }
            }
        });

        if let Ok(entries) = &mut tab.entries {
            let mut new_path = None;

            let ctx = ui.ctx().clone();

            TableBuilder::new(ui)
                .column(Column::remainder())
                .column(Column::auto().at_least(160.))
                .column(Column::auto().at_least(60.))
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
                    if let Some((name, is_dir)) = &mut tab.state.add_entry {
                        let mut close = false;
                        body.row(18.0, |mut row| {
                            row.set_selected(true);
                            row.col(|ui| {
                                let resp = TextEdit::singleline(name)
                                    .return_key(Some(egui::KeyboardShortcut::new(
                                        Modifiers::NONE,
                                        Key::Enter,
                                    )))
                                    .cursor_at_end(true)
                                    .desired_width(ui.available_width())
                                    .show(ui);
                                
                                if resp.response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    close = true;
                                } else {
                                    resp.response.request_focus();
                                }
                            });
                            row.col(|ui| {});
                            row.col(|ui| {});
                        });
                        if close {
                            let path = Path::new(&tab.path).join(name);
                            if *is_dir {
                                let _ = std::fs::create_dir_all(path);
                            } else {
                                let _ = std::fs::File::create(path);
                            }
                            tab.state.relead = true;
                            tab.state.add_entry = None;
                        }
                    }

                    for (i, entry) in entries.iter().enumerate() {
                        if tab.search.is_empty()
                            || entry
                                .file_name
                                .to_lowercase()
                                .contains(&tab.search.to_lowercase())
                        {
                            body.row(18.0, |mut row| {
                                row.set_selected(tab.selected_entries.contains(&i));
                                row.col(|ui| {
                                    let mut text: egui::RichText =
                                        entry.file_name.to_string().into();
                                    if entry.file_type.is_dir() {
                                        text = text.strong();
                                    }
                                    Label::new(text).selectable(false).ui(ui);
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

                                let command = ctx.input(|i| i.modifiers.command);
                                let shift = ctx.input(|i| i.modifiers.shift);
                                if resp.clicked() {
                                    if shift {
                                        if let Some(first) = tab.last_clicked_entry {
                                            if first >= i {
                                                for x in i..first {
                                                    tab.selected_entries.insert(x);
                                                }
                                            } else {
                                                for x in first + 1..=i {
                                                    tab.selected_entries.insert(x);
                                                }
                                            }
                                        }
                                    } else if command {
                                        if tab.selected_entries.contains(&i) {
                                            tab.selected_entries.remove(&i);
                                        } else {
                                            tab.selected_entries.insert(i);
                                        }
                                    } else {
                                        if tab.selected_entries.contains(&i) {
                                            tab.selected_entries.clear();
                                        } else {
                                            tab.selected_entries.clear();
                                            tab.selected_entries.insert(i);
                                        }
                                    }
                                    tab.last_clicked_entry = Some(i);
                                }

                                let is_main = tab.selected_entries.is_empty();
                                let action_entries: Vec<_> = if is_main {
                                    vec![tab.info.as_ref().unwrap()]
                                } else {
                                    entries
                                        .iter()
                                        .enumerate()
                                        .filter(|(i, x)| tab.selected_entries.contains(i))
                                        .map(|(i, x)| x)
                                        .collect()
                                };
                                resp.context_menu(|ui| {
                                    for action in self.actions.iter() {
                                        if (action.can_execute)(&action_entries, is_main) {
                                            if ui.button((action.name)(&action_entries)).clicked() {
                                                for entry in action_entries.iter() {
                                                    (action.execute)(&entry, &mut tab.state);
                                                }
                                                ui.close_menu();
                                            }
                                        }
                                    }
                                });
                            });
                        }
                    }
                });

            if let Some(new_path) = new_path {
                tab.refresh(new_path);
            }
        }

        if tab.state.relead {
            tab.refresh_hard(tab.path.clone());
        }
    }
}
