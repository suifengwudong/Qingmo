mod app;
mod cli;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // If the first non-binary argument is "llm", enter CLI mode.
    // run_cli handles all errors internally: it prints to stderr and calls
    // std::process::exit(1) on failure, so the `return Ok(())` below is only
    // reached on success.
    if args.get(1).map(|s| s.as_str()) == Some("llm") {
        cli::run_cli(&args[2..]);
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("清墨")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "清墨",
        options,
        Box::new(|cc| Ok(Box::new(app::TextToolApp::new(cc)))),
    )
}
