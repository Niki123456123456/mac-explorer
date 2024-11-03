use crate::files::FileEntry;

pub struct Action {
    pub name: Box<dyn Fn(&FileEntry) -> String>,
    pub can_execute: Box<dyn Fn(&FileEntry) -> bool>,
    pub execute: Box<dyn Fn(&FileEntry)>,
}

impl Action {
    pub fn new(
        name: impl Fn(&FileEntry) -> String + 'static,
        can_execute: impl Fn(&FileEntry) -> bool + 'static,
        execute: impl Fn(&FileEntry) + 'static,
    ) -> Self {
        Self {
            name: Box::new(name),
            can_execute: Box::new(can_execute),
            execute: Box::new(execute),
        }
    }
}

pub fn actions()-> Vec<Action>{
    let actions = vec![];

    return actions;
}