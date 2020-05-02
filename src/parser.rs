use std::{
    io,
    sync::mpsc::{Receiver, Sender},
};
use termion::event::Key;
use crate::Control;

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
    done: Receiver<Control>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Result<Key, io::Error>>,
{
    pub(crate) fn new(source: I, done: Receiver<Control>, output: Sender<Parsed>) -> Self {
        Parser {
            source,
            output,
            done,
        }
    }

    pub(crate) fn run(self) -> Result<(), std::sync::mpsc::SendError<Parsed>> {
        for symbol in self.source {
            match self.done.try_recv() {
                Ok(Control::Stop) => break,
                _ => ()
            }

            match symbol.expect("Failed to parse symbol") {
                Key::Ctrl(c) if c == 'c' => {
                    self.output.send(Parsed::Stop)?;
                    break;
                }
                Key::Backspace => self.output.send(Parsed::Backspace)?,
    //            Key::Char('\n') => self.output.send(Parsed::Enter)?,
                Key::Char(c) => self.output.send(Parsed::Symbol(c))?,
                _ => {}
            }
        }
        Ok(())
    }
}
