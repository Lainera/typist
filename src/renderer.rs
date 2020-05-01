use std::{
    io::{self, Stdout, prelude::*},
    sync::mpsc::Receiver,
};
use termion::raw::{
    RawTerminal,
    IntoRawMode,
};

use crate::Control;

pub(crate) struct Renderer {
    stdout: RawTerminal<Stdout>,
    input: Receiver<Control>
}

impl Renderer {
   pub(crate) fn new(stdout: Stdout, input: Receiver<Control>) -> Result<Self, io::Error> {
        let stdout = stdout.into_raw_mode()?;
        Ok(Renderer {
            stdout,
            input,
        })
    }

   pub(crate) fn run(mut self) -> Result<(), io::Error> {
        for c in self.input {
            match c {
                Control::Stop => {
                    // Restore everything back to normal on Stop command;
                    write!(self.stdout, "{}", termion::cursor::Restore)?;
                    self.stdout.flush()?;
                    break
                },
                Control::Backspace => write!(
                    self.stdout, 
                    "{} {}", 
                    termion::cursor::Left(1), 
                    termion::cursor::Left(1)
                )?,
                Control::Enter => write!(
                    self.stdout, 
                    "\n{}", 
                    termion::cursor::Left(std::u16::MAX)
                )?,
                Control::Symbol(result) => match result {
                    Ok(s) => write!(
                        self.stdout, 
                        "{}{}{}", 
                        termion::color::Fg(termion::color::Green), 
                        s, 
                        termion::color::Fg(termion::color::Reset)
                    )?,
                    Err(s) => write!(
                        self.stdout, 
                        "{}{}{}", 
                        termion::color::Fg(termion::color::Red), 
                        s,
                        termion::color::Fg(termion::color::Reset),
                    )?,
                },
            }
            self.stdout.flush()?
        }
        Ok(())
    }
}

