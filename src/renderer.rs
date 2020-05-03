use std::{
    io::{self, prelude::*, Stdout},
    sync::mpsc::Receiver,
};
use termion::raw::{IntoRawMode, RawTerminal};

use crate::Control;

pub(crate) fn draw_initial(mut stdout: RawTerminal<Stdout>, word: &str) -> Result<(), io::Error> {
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )?;

    for line in word.lines() {
        write!(stdout, "{}\n{}", line, termion::cursor::Left(std::u16::MAX))?
    }

    write!(
        stdout,
        "{}{}{}",
        termion::cursor::Goto(1, 1),
        termion::cursor::Show,
        termion::cursor::BlinkingUnderline
    )?;
    stdout.flush()
}

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
                Control::Backspace(symbol) => write!(
                    self.stdout,
                    "{}{}{}",
                    termion::cursor::Left(1),
                    symbol,
                    termion::cursor::Left(1)
                )?,
                Control::GoTo((row, column)) => write!(
                    self.stdout,
                    "{}",
                    // Termion is one-based, not zero based
                    termion::cursor::Goto((column + 1) as u16, (row + 1) as u16)
                )?,
                Control::Enter => {
                    write!(self.stdout, "\n{}", termion::cursor::Left(std::u16::MAX))?
                }
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
