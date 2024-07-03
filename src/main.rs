use std::{
    io::{self, Read, Write},
    path::PathBuf,
    sync::{mpsc::channel, Arc, Mutex}, thread, time::Duration,
};

use app::App;
use clap::Parser;
use config::Config;
use crossterm::tty::IsTty;
use log::info;

mod app;
mod tui;
mod config;

#[derive(Parser)]
struct Args {
    pub data: Option<PathBuf>,
}

struct Log(Arc<Mutex<Vec<u8>>>);

impl Log {
    fn new() -> Self {
        Log(Arc::default())
    }

    fn inspect(&self, f: impl FnOnce(&[u8])) {
        f(&self.0.lock().unwrap())
    }
}

impl Write for Log {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.lock().unwrap().flush()
    }
}

impl Clone for Log {
    fn clone(&self) -> Self {
        Log(self.0.clone())
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let config = Config::default();

    let log = Log::new();
    env_logger::builder()
        .target(env_logger::Target::Pipe(Box::new(log.clone())))
        .filter_level(log::LevelFilter::max())
        .init();

    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = tui::restore();

        hook(panic_info)
    }));

    let mut terminal = tui::init()?;

    let input = match args.data {
        Some(path) => std::fs::read_to_string(path)?,
        None => if io::stdin().is_tty() {
            String::new()
        } else {
            let (tx, rx) = channel();
            thread::spawn(move || {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).unwrap();
                tx.send(input).unwrap();
            });
            match rx.recv_timeout(Duration::from_secs(3)) {
                Ok(input) => input,
                Err(_) => String::new(),
            }
        },
    };
    let mut app = App::new(input, config);

    info!("Running application");

    app.run(&mut terminal)?;

    info!("Application exit");

    tui::restore()?;

    log.inspect(|data| {
        io::stdout().write_all(data).unwrap();
        io::stdout().flush().unwrap();
    });

    Ok(())
}
