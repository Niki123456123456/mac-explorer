use std::{collections::HashSet, io};

use crate::{actions::ActionState, files::{get_entries, get_meta, FileEntry}};

#[derive(Debug)]
pub struct Tab {
    pub id: egui::Id,
    pub path: String,
    pub search: String,
    pub info: io::Result<FileEntry>,
    pub entries: Result<Vec<FileEntry>, io::Error>,
    pub selected_entries: HashSet<usize>,
    pub last_clicked_entry: Option<usize>,
    pub previous_paths: Vec<String>,
    pub previous_paths2: Vec<String>,
    pub state: ActionState,
}

impl Tab {
    pub fn new2(path: impl Into<String>, id: egui::Id) -> Self {
        let path = path.into();
        let info = get_meta(&path);
        let entries = get_entries(&path);
        return Self {
            id,
            path,
            search: "".into(),
            info,
            entries,
            previous_paths: vec![],
            previous_paths2: vec![],
            selected_entries: Default::default(),
            last_clicked_entry: None,
            state: ActionState::default(),
        };
    }

    pub fn refresh(&mut self, path: impl Into<String>) {
        let path = path.into();
        if let Ok(i) = &self.info {
            if i.path == path {
                return;
            }
        }

        self.refresh_hard(path);
    }

    pub fn refresh_hard(&mut self, path: impl Into<String>) {
        let mut new = Self::new2(path, self.id);
        new.previous_paths.append(&mut self.previous_paths);
        new.previous_paths.push(self.path.clone());
        *self = new;
    }
}
