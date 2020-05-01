use std::{
    sync::mpsc::{
        Sender,
        Receiver,
    },
    io
};
use termion::event::Key;

pub (crate) enum Parsed {
    Stop,
    Backspace,
    Enter,
    Symbol(char),
}

pub struct Parser<I> where 
 I: Iterator<Item = Result<Key, io::Error>> 
{
    source: I,
    output: Sender<Parsed>,
    done: Receiver<()>,
}

impl <I>Parser<I> where 
    I: Iterator<Item = Result<Key, io::Error>>,
{
    pub(crate) fn new(source: I, done: Receiver<()>, output: Sender<Parsed>) -> Self {
        Parser{
            source, 
            output,
            done,
        }
    } 
    
    pub(crate) fn run(self) -> Result<(), std::sync::mpsc::SendError<Parsed>> {
        for symbol in self.source {
            if let Ok(_) = self.done.try_recv() { break }
            match symbol.expect("Failed to parse symbol") {
                Key::Ctrl(c) if c == 'c' => {
                    self.output.send(Parsed::Stop)?;
                    break
                },
                Key::Backspace => self.output.send(Parsed::Backspace)?,
                Key::Char('\n') => self.output.send(Parsed::Enter)?,
                Key::Char(c) => self.output.send(Parsed::Symbol(c))?,
                _ => {},
            }
        }
        Ok(())
    }
}
