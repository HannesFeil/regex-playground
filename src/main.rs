use std::io;

use app::App;
use clap::Parser;

mod app;
mod tui;

#[derive(Parser)]
struct Args {}

fn main() -> io::Result<()> {
    Args::parse();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = tui::restore();

        hook(panic_info)
    }));

    let mut app = App::new();

    let mut terminal = tui::init()?;

    app.run(&mut terminal);

    tui::restore()?;

    Ok(())
}
