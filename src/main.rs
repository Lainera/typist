use std::{
    io::{
        self,
        prelude::*,
    },
    thread
};
use std::sync::mpsc::{
    Sender,
    Receiver,
    channel,
};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
// local
mod control;
mod parser;
mod renderer;

use renderer::Renderer;
use control::Control;
use parser::{
    Parser,
    Parsed
};

//moves around string,
//new (&str)? yes, I think makes sense to pass pointer to string and have internal cursor
//listens to input from parser, parser sends parsed, checker receives parsed, checks and forwards
//Checked to renderer, which in return draws on the terminal.
//Checker also tells parser to stop listening when done.

pub (crate) struct Checker<'a>
{
    contents: &'a str, 
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: Sender<()>,
}

impl <'a>Checker<'a> {
    pub(crate) fn new(input: Receiver<Parsed>, output: Sender<Control>, done: Sender<()>) -> Self {
        let contents = "hue hue";
        Checker{
            input,
            output,
            contents,
            done,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), std::sync::mpsc::SendError<Control>> {
        println!("Keque");
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // or print word before converting terminal into raw mode?
    let word = std::fs::read_to_string("./some-text.txt")?;

    let mut stdout = io::stdout().into_raw_mode().expect("failed to convert to raw mode");
    write!(
        stdout, 
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::BlinkingUnderline
    ).expect("Failed to set up terminal");
    // init communication channels
    let (tx_done, rx_done): (Sender<()>, Receiver<()>) = channel();
    let (tx_parsed, rx_parsed): (Sender<Parsed>, Receiver<Parsed>) = channel();
    let (tx_checked, rx_checked): (Sender<Control>, Receiver<Control>) = channel();
    // init subprocesses
    let parser = Parser::new(io::stdin().keys(), rx_done, tx_parsed);    
    let checker = Checker::new(rx_parsed, tx_checked, tx_done); 
    let renderer = Renderer::new(io::stdout(), rx_checked)?;

    // probably somewhere here need to print initial string with renderer.
    thread::spawn(move || checker.run());
    let renderer_handle = thread::spawn(move || renderer.run());
    parser.run()?;
    renderer_handle.join()
        .expect("Renderer thread panicked")
        .expect("Renderer failed to flush");

    Ok(())
}
