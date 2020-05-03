use std::sync::mpsc::{channel, Receiver, Sender};
use std::{io, thread};

use termion::input::TermRead;
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

    // Init communication channels
    // Done channel for remote parser shutdown.
    let (tx_done, rx_done): (Sender<Control>, Receiver<Control>) = channel();
    // Parsed to connect Parser with Checker
    let (tx_parsed, rx_parsed): (Sender<Parsed>, Receiver<Parsed>) = channel();
    // Checked to connect Checker with Renderer
    let (tx_checked, rx_checked): (Sender<Control>, Receiver<Control>) = channel();

    // Start renderer, draw initial view, move into it's own thread;
    let mut renderer = Renderer::new(io::stdout(), rx_checked)?;
    renderer.draw_initial(&source_text)?;
    let renderer_handle = thread::spawn(move || renderer.run());

    // Start checker, move into it's own thread.
    let checker = Checker::new(rx_parsed, tx_checked, tx_done, source_text);
    thread::spawn(move || checker.run());

    // Start parser, listen to stdin
    let parser = Parser::new(io::stdin().keys(), rx_done, tx_parsed);
    parser.run()?;
    // wait for renderer to cleanup
    renderer_handle
        .join()
        .expect("Renderer thread panicked")
        .expect("Renderer failed to flush");

    Ok(())
}
