use std::sync::mpsc::{channel, Receiver, Sender};
use std::{io, thread};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
// local
mod checker;
mod parser;
mod renderer;

use checker::{Checker, Control};
use parser::{Parsed, Parser};
use renderer::Renderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source_text = if let Some(path) = std::env::args().nth(1) {
        std::fs::read_to_string(&path)?
    } else {
        String::from("Hey dawg, this \nis a test string \nfor you to practice!")
    };

    let stdout = io::stdout()
        .into_raw_mode()
        .expect("failed to convert to raw mode");

    renderer::draw_initial(stdout, &source_text)?;
    // init communication channels
    // Done to communicate early exit via ctrl-c
    let (tx_done, rx_done): (Sender<Control>, Receiver<Control>) = channel();
    // Parsed to connect Parser with Checker
    let (tx_parsed, rx_parsed): (Sender<Parsed>, Receiver<Parsed>) = channel();
    // Checked to connect Checker with Renderer
    let (tx_checked, rx_checked): (Sender<Control>, Receiver<Control>) = channel();

    // init subprocesses
    let parser = Parser::new(io::stdin().keys(), rx_done, tx_parsed);
    let checker = Checker::new(rx_parsed, tx_checked, tx_done, source_text);
    let renderer = Renderer::new(io::stdout(), rx_checked)?;

    // probably somewhere here need to print initial string with renderer.
    thread::spawn(move || checker.run());
    let renderer_handle = thread::spawn(move || renderer.run());
    // start listening to stdin
    parser.run()?;
    renderer_handle
        .join()
        .expect("Renderer thread panicked")
        .expect("Renderer failed to flush");

    Ok(())
}
