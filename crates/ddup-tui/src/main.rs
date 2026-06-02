//! ddup terminal UI.

mod app;
mod events;
mod modal;
mod tree;
mod tui_runtime;
mod ui;

use clap::Parser;

use crate::app::{AppState, Args};

#[allow(clippy::print_stdout)]
fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let _ = AppState::new(&args);
    println!("ddup tui");
}
