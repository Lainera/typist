use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};
use termion::event::Key;

pub(crate) enum Parsed {
    Stop,
    Backspace,
    Symbol(char),
}

pub struct Parser<I>
where
    I: Iterator<Item = Result<Key, io::Error>>,
{
    source: I,
    output: Sender<Parsed>,
    done: Receiver<()>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Result<Key, io::Error>>,
{
    pub(crate) fn new(source: I, done: Receiver<()>, output: Sender<Parsed>) -> Self {
        Parser {
            source,
            output,
            done,
        }
    }
    pub(crate) fn run(mut self) -> Result<(), std::sync::mpsc::SendError<Parsed>> {
        loop {
            if let Ok(_) = self.done.try_recv() {
                break;
            }
            if let Some(symbol) = self.source.next() {
                match symbol.expect("Failed to parse symbol") {
                    Key::Ctrl(c) if c == 'c' => {
                        self.output.send(Parsed::Stop)?;
                        break;
                    }
                    Key::Char(c) if c == '\t' => {
                        for _ in 0..4 {
                            self.output.send(Parsed::Symbol(' '))?
                        }
                    }
                    Key::Backspace => self.output.send(Parsed::Backspace)?,
                    Key::Char(c) => self.output.send(Parsed::Symbol(c))?,
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
