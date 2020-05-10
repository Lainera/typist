use std::{
    io::{self, prelude::*, Stdout},
    sync::{
        mpsc::Receiver,
        Arc,
    },
};
use termion::raw::{IntoRawMode, RawTerminal};

use crate::{Control, Source};

pub(crate) struct Renderer {
    stdout: RawTerminal<Stdout>,
    input: Receiver<Control>,
    source: Arc<Source>,
    window_size: usize,
    row_modifier: usize,
}

// Try out passing strings separately,
// Figure out whether it would even work.
impl Renderer {
    pub(crate) fn new(stdout: Stdout, input: Receiver<Control>, source: Arc<Source>) -> Result<Self, io::Error> {
        let stdout = stdout.into_raw_mode()?;
        Ok(Self {
            stdout,
            input,
            source,
            window_size: 2,
            row_modifier: 0,
        })
    }

    pub(crate) fn run(mut self) -> Result<(), io::Error> {
        self.draw_initial()?;
        let window_size = self.window_size;
        let mut row_modifier = self.row_modifier;
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
                Control::Previous(None, (row, column)) => {
                    if row < window_size - 1 {
                        write!(
                            self.stdout,
                            "{}",
                            // Termion is one-based, not zero based
                            termion::cursor::Goto((column + 1) as u16, (row + 1) as u16)
                        )?
                    } else {
                        row_modifier -= 1;
                        write!(
                            self.stdout,
                            "{}{}",
                            termion::scroll::Down(1),
                            termion::cursor::Goto(
                                (column + 1) as u16,
                                (row - row_modifier + 1) as u16
                            ),
                        )?
                    }
                }
                Control::Next(None, (row, column)) => {
                    if row < window_size {
                        write!(
                            self.stdout,
                            "{}",
                            // Termion is one-based, not zero based
                            termion::cursor::Goto((column + 1) as u16, (row + 1) as u16)
                        )?
                    } else {
                        row_modifier += 1;
                        write!(
                            self.stdout,
                            "{}\rfor you to practice!{}",
                            termion::scroll::Up(1),
                            termion::cursor::Goto(0, (row - row_modifier + 1) as u16)
                        )?
                    }
                }
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
    
    fn draw_line(&mut self, n: usize) -> Result<(), io::Error> {
        if let Some(line) = self.source.get_line(n) {
            let to_write = line.iter().fold(String::new(), |mut acc, &c| {
                acc.push(c.clone());
                acc
            });
            write!(self.stdout, "{}\r\n", to_write)?;
        }
        Ok(())
    }

    pub(crate) fn draw_initial(&mut self) -> Result<(), io::Error> {
        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide
        )?;

        for line in 0..2 {
           self.draw_line(line)?; 
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
