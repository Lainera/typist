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
    cursor: (usize, usize),
}

fn absolute_difference(a: usize, b: usize) -> u32 {
   let i = a as i32 - b as i32;
    if i < 0 {
        -i as u32
    } else {
        i as u32
    }
}

impl Renderer {
    pub(crate) fn new(stdout: Stdout, input: Receiver<Control>, source: Arc<Source>) -> Result<Self, io::Error> {
        let stdout = stdout.into_raw_mode()?;
        Ok(Self {
            stdout,
            input,
            source,
            window_size: 2,
            cursor: (0, 0)
        })
    }

    pub(crate) fn run(mut self) -> Result<(), io::Error> {
        self.draw_initial()?;
        let window_size = self.window_size;
        let (mut head, mut tail) = self.cursor;
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
                    if tail > 0 && head > 0 && absolute_difference(head, tail) == 0 {
                        tail -= 1;
                        head -= 1;
                        let line = self.source.get_line(tail).unwrap();
                        let as_str = line.iter().fold(String::new(), |mut acc, &c| {
                            acc.push(c);
                            acc
                        });
                        write!(
                            self.stdout,
                            "{}{}{}{}",
                            termion::scroll::Down(1),
                            termion::cursor::Goto(1, 1),
                            as_str,
                            termion::cursor::Goto(
                                (column + 1) as u16,
                                (row - tail + 1) as u16
                            ),
                        )?
                    } else {
                        head -= 1;
                        write!(
                            self.stdout,
                            "{}",
                            termion::cursor::Goto((column + 1) as u16, (row - tail + 1) as u16)
                        )?;
                    } 
                }
                Control::Next(None, (row, column)) => {
                    if absolute_difference(tail, head + 1) < window_size as u32 {
                        head += 1;
                        write!(
                            self.stdout,
                            "{}",
                            termion::cursor::Goto((column + 1) as u16, (row - tail + 1) as u16)
                        )?;
                    } else {
                        if let Some(line) = self.source.get_line(head + 1) {
                            let as_str = line.iter().fold(String::new(), |mut acc, &c| {
                                acc.push(c);
                                acc
                            });
                            
                            head += 1;
                            tail += 1;
                            
                            write!(
                                self.stdout,
                                "{}\r{}{}",
                                termion::scroll::Up(1),
                                as_str,
                                termion::cursor::Goto((column + 1) as u16, (row - tail + 1) as u16)
                                )?;

                        } else {
                            write!(
                                self.stdout,
                                "{}",
                                termion::cursor::Goto(1, (row - tail + 1) as u16)
                            )?
                        };
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

        for line in 0..self.window_size {
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
