use crate::files::{FileEntry, Restriction};

pub trait GetName: Fn(&Vec<&FileEntry>) -> String {}
impl<F> GetName for F where F: Fn(&Vec<&FileEntry>) -> String {}

pub trait CanExecute: Fn(&Vec<&FileEntry>, bool) -> bool {}
impl<F> CanExecute for F where F: Fn(&Vec<&FileEntry>, bool) -> bool {}

pub trait Execute: Fn(&FileEntry, &mut ActionState) {}
impl<F> Execute for F where F: Fn(&FileEntry, &mut ActionState) {}

#[derive(Default, Debug)]
pub struct ActionState {
    pub relead: bool,
    pub add_entry : Option<(String, bool)>,
}

pub struct Action {
    pub name: Box<dyn GetName>,
    pub can_execute: Box<dyn CanExecute>,
    pub execute: Box<dyn Execute>,
}

impl Action {
    pub fn new(
        name: impl GetName + 'static,
        can_execute: impl CanExecute + 'static,
        execute: impl Execute + 'static,
    ) -> Self {
        Self {
            name: Box::new(name),
            can_execute: Box::new(can_execute),
            execute: Box::new(execute),
        }
    }

    pub fn constant(
        display_name: &'static str,
        restriction: Restriction,
        execute: impl Execute + 'static,
    ) -> Self {
        return Self::new(
            |e| display_name.to_string(),
            move |e, b| e.iter().all(|e| e.fullfills(&restriction, b)),
            execute,
        );
    }

    pub fn open_with(
        app_name: &'static str,
        display_name: &'static str,
        restriction: Restriction,
    ) -> Self {
        return Self::new(
            |e| display_name.to_string(),
            move |e, b| e.iter().all(|e| e.fullfills(&restriction, b)),
            move |e, s| {
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
    actions.push(Action::constant("add file",Restriction::Main, |e, s|{
        s.add_entry = Some(("".into(), false));
    }));
    actions.push(Action::constant("add dir", Restriction::Main, |e, s|{
        s.add_entry = Some(("".into(), true));
    }));
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
    actions.push(Action::open_with("Finder", "finder", Restriction::Folder));
    actions.push(Action::open_with("Zed", "zed", Restriction::None));
    actions.push(Action::constant("delete", Restriction::Not(Box::new(Restriction::Main)), |e, s|{
        if e.file_type.is_file() {
            let _ =std::fs::remove_file(e.path.clone());
        }
        if e.file_type.is_dir() {
            let _ =std::fs::remove_dir_all(e.path.clone());
        }
        s.relead = true;
    }));
    return actions;
}
