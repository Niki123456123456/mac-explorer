use crate::files::{FileEntry, Restriction};

pub trait GetName: Fn(&Vec<&FileEntry>) -> String {}
impl<F> GetName for F where F: Fn(&Vec<&FileEntry>) -> String {}

pub trait CanExecute: Fn(&Vec<&FileEntry>) -> bool {}
impl<F> CanExecute for F where F: Fn(&Vec<&FileEntry>) -> bool {}

pub struct Action {
    pub name: Box<dyn GetName>,
    pub can_execute: Box<dyn CanExecute>,
    pub execute: Box<dyn Fn(&FileEntry)>,
}

impl Action {
    pub fn new(
        name: impl GetName + 'static,
        can_execute: impl CanExecute + 'static,
        execute: impl Fn(&FileEntry) + 'static,
    ) -> Self {
        Self {
            name: Box::new(name),
            can_execute: Box::new(can_execute),
            execute: Box::new(execute),
        }
    }

    pub fn open_with(
        app_name: &'static str,
        display_name: &'static str,
        restriction: Restriction,
    ) -> Self {
        return Self::new(
            |e| display_name.to_string(),
            move |e| e.iter().all(|e|e.fullfills(restriction)),
            move |e| {
                let _ = std::process::Command::new("open")
                    .arg("-a")
                    .arg(app_name)
                    .arg(&e.path)
                    .status();
            },
        );
    }
}

pub fn actions() -> Vec<Action> {
    let mut actions = vec![];

    /*
     if ui.button("make executable").clicked() {
                                        std::process::Command::new("chmod")
                                            .arg("755")
                                            .arg(entry.path.clone())
                                            .status();
                                        ui.close_menu();
                                    }
     */

    actions.push(Action::open_with(
        "Visual Studio Code",
        "vscode",
        Restriction::None,
    ));
    actions.push(Action::open_with(
        "Terminal",
        "terminal",
        Restriction::Folder,
    ));
    return actions;
}
