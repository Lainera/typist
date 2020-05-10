use std::{
    io::{self, prelude::*, Stdout},
    sync::mpsc::Receiver,
};
use termion::raw::{IntoRawMode, RawTerminal};

use crate::Control;

pub(crate) struct Renderer {
    stdout: RawTerminal<Stdout>,
    input: Receiver<Control>,
}

impl Renderer {
    pub(crate) fn new(stdout: Stdout, input: Receiver<Control>) -> Result<Self, io::Error> {
        let stdout = stdout.into_raw_mode()?;
        Ok(Self { stdout, input })
    }
    
    pub(crate) fn run(mut self) -> Result<(), io::Error> {
        for c in self.input {
            match c {
                Control::Stop => {
                    // Restore everything back to normal on Stop command;
                    write!(self.stdout, "{}", termion::cursor::Restore)?;
                    self.stdout.flush()?;
                    break;
                }
                Control::Previous(Some(symbol), _) => write!(
                    self.stdout,
                    "{}{}{}",
                    termion::cursor::Left(1),
                    symbol,
                    termion::cursor::Left(1)
                )?,
                Control::Previous(None, (row, column)) | Control::Next(None, (row, column)) => write!(
                    self.stdout,
                    "{}",
                    // Termion is one-based, not zero based
                    termion::cursor::Goto((column + 1) as u16, (row + 1) as u16)
                )?,
                Control::Next(Some(result), _) => match result {
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

    pub(crate) fn draw_initial(&mut self, word: &str) -> Result<(), io::Error> {
        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        )?;

        for line in word.lines() {
            write!(self.stdout, "{}\n\r", line)?
        }

        write!(
            self.stdout,
            "{}{}{}",
            termion::cursor::Goto(1, 1),
            termion::cursor::Show,
            termion::cursor::BlinkingUnderline
        )?;

        self.stdout.flush()
    }
}
