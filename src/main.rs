pub mod app;

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
