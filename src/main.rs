pub mod app;
pub mod actions;
pub mod files;
pub mod tab;
pub mod tabviewer;
pub mod zip;

fn main() {
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "mac-explorer",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    ).unwrap();
}
