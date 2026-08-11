//! `fepdf-layout-ui`: Native GUI desktop entry point for fepdf-layout.

pub mod app;

use app::FepdfLayoutApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("fepdf-layout — 帳票エディタ")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "fepdf-layout",
        native_options,
        Box::new(|cc| Ok(Box::new(FepdfLayoutApp::new(cc)))),
    )
}
