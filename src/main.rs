use std::sync::mpsc::{channel, Receiver, Sender};
use std::{
    io,
    thread,
};

use termion::input::TermRead;
use termion::raw::IntoRawMode;
// local
mod control;
mod parser;
mod renderer;

use control::Control;
use parser::{Parsed, Parser};
use renderer::Renderer;

/// What are implementation options? Since we are traversing string, we cannot index into it (to account for grapheme clusters).
/// Which leaves us with option of using
/// - Internal iterator: good for advancing, but not that great for rewinding, since iter.nth(n)
/// consumes all items up until n. Meaning going back would be really inefficient.
/// - Split input string by chars and collect into vector, keeping cursor. Then we can index into
/// that vector, and easily go back.
/// Going with Vec.
/// API  
///
/// `next() -> Option<char>` => advances the contents. 
/// `prev() -> Option<char>` => rewinds the contents one symbol at a time.
/// `run(mut self) -> Result<(), std::sync::mpsc::SendError<Control>>` => reads input from parser,
/// comparing it with w/e we have in the contents.
pub(crate) struct Checker {
    source: StringCursor,
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: Sender<Control>,
}

/// Helper struct that moves around source string
struct StringCursor {
    source: Vec<char>,
    cursor: usize,
}

impl StringCursor {
    fn new<S: Into<String>>(source: S) -> Self {
        let source = source
            .into()
            .chars()
            .collect::<Vec<char>>();

        Self {
            source,
            cursor: 0
        }
    }
    // Need to include some more info here. Potentially line number and length. 
    fn next(&mut self) -> Option<&char> {
        let symbol = self.source.get(self.cursor);
        if symbol.is_some() {
            self.cursor += 1;
        }
        symbol
    }
    
    fn prev(&mut self) -> Option<&char> {
        // Beginning of source? -> Nowhere to run
        if self.cursor.checked_sub(1).is_none() {
            None
        } else {
        // Otherwise rewind cursor one position and spit out contents. 
            self.cursor -= 1;
            self.source.get(self.cursor)
        }
    }
}

impl Checker {
    pub(crate) fn new(
        input: Receiver<Parsed>, 
        output: Sender<Control>, 
        done: Sender<Control>, 
        source_string: String
    ) -> Self {
        let source = StringCursor::new(source_string); 
        Self {
            done,
            input,
            output,
            source,
        }
    }

    pub(crate) fn run(self) -> Result<(), std::sync::mpsc::SendError<Control>> {
        let mut source = self.source;
        for parsed in self.input {
            match parsed {
                Parsed::Stop => {
                    self.output.send(Control::Stop)?;
                    break;
                },
                Parsed::Backspace => {
                    match source.prev() {
                        Some(&source_symbol) => {
                            // Maybe need to handle variety of other newline chars?
                            if source_symbol == '\n' {
                                self.output.send(Control::PreviousLine)?
                            } else {
                                self.output.send(Control::Backspace(source_symbol))?
                            }
                        },
                        // We are at first symbol in source string
                        None => (),
                    }
                },
                Parsed::Symbol(symbol) => {
                   match source.next() {
                       Some(&source_symbol) if source_symbol == '\n' => {
                           if source_symbol == symbol {
                               self.output.send(Control::Enter)?
                           } else {
                               source.prev();
                           }
                       },
                       Some(&source_symbol) => {
                           if source_symbol == symbol {
                               self.output.send(Control::Symbol(Ok(symbol)))?
                           } else {
                               self.output.send(Control::Symbol(Err(symbol)))?
                           }
                       },
                       None => {
                           self.output.send(Control::Stop)?;
                           self.done.send(Control::Stop)?;
                           break;
                       }
                   }
                },
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let word = std::fs::read_to_string("./some-text.txt")?;
    let stdout = io::stdout()
        .into_raw_mode()
        .expect("failed to convert to raw mode");

    renderer::draw_initial(stdout, &word)?;
    // init communication channels
    // Done to communicate early exit via ctrl-c
    let (tx_done, rx_done): (Sender<Control>, Receiver<Control>) = channel();
    // Parsed to connect Parser with Checker
    let (tx_parsed, rx_parsed): (Sender<Parsed>, Receiver<Parsed>) = channel();
    // Checked to connect Checker with Renderer
    let (tx_checked, rx_checked): (Sender<Control>, Receiver<Control>) = channel();

    // init subprocesses
    let parser = Parser::new(io::stdin().keys(), rx_done, tx_parsed);
    let checker = Checker::new(rx_parsed, tx_checked, tx_done, word);
    let renderer = Renderer::new(io::stdout(), rx_checked)?;

    // probably somewhere here need to print initial string with renderer.
    thread::spawn(move || checker.run());
    let renderer_handle = thread::spawn(move || renderer.run());
    parser.run()?;
    renderer_handle
        .join()
        .expect("Renderer thread panicked")
        .expect("Renderer failed to flush");

    Ok(())
}
