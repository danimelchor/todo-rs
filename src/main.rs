use anyhow::Result;
use rust_todo::configuration::get_configuration;
use rust_todo::{app::App, cli, ui};

fn main() -> Result<()> {
    // Check args, if none, run ui, else run cli
    let settings = get_configuration();
    let app = App::new(settings);

    if std::env::args().len() > 1 {
        cli::start_cli(app)
    } else {
        ui::start_ui(app)
    }
}