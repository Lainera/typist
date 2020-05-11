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
    window_size: u16,
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
        let (_, window_size) = termion::terminal_size()?;
        let stdout = stdout.into_raw_mode()?;
        Ok(Self {
            stdout,
            input,
            source,
            // last line is not used
            window_size: window_size - 1,
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
                // Moving cursor horizontally within same line.
                Control::Previous(Some(symbol), _) => write!(
                    self.stdout,
                    "{}{}{}",
                    termion::cursor::Left(1),
                    symbol,
                    termion::cursor::Left(1)
                )?,
                // At the beginning of the line and need to move cursor up
                Control::Previous(None, (row, column)) => {
                    if tail > 0 && head > 0 && absolute_difference(head, tail) == 0 {
                    // need to scroll up because head == tail and we are not at the start yet.
                    // Check if line still exists in the source, then move head and tail up.
                        tail -= 1;
                        head -= 1;
                    // Need to request line as collection of chars, and errors for that line from
                    // checker, merge both into colored line.
                        let line = self.source.get_line(tail).unwrap();
                        let as_str = line.iter().fold(String::new(), |mut acc, &c| {
                            acc.push(c);
                            acc
                        });
                    // 0 < tail < row; To get actual position on the screen we need to subtract
                    // tail from row. Or just go to 1, that also works, because we just scrolled
                    // Down one line, printed contents and have to put cursor at the end of that
                    // line.
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
                    // backspacing to previous line, but we are not at the very first line yet, so
                    // no need to move tail up. Update head and proceed.
                        head -= 1;
                        write!(
                            self.stdout,
                            "{}",
                            termion::cursor::Goto((column + 1) as u16, (row - tail + 1) as u16)
                        )?;
                    } 
                }
                // At the end of the line
                Control::Next(None, (row, column)) => {
                    if absolute_difference(tail, head + 1) < window_size as u32 {
                        // Haven't exceeded window size, jump onto next line.
                        head += 1;
                        write!(
                            self.stdout,
                            "{}",
                            termion::cursor::Goto((column + 1) as u16, (row - tail + 1) as u16)
                        )?;
                    } else {
                        // Exceeded window size, and
                        if let Some(line) = self.source.get_line(head + 1) {
                            // there is more! get next line, draw, update counters. 
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
                            // There is no more lines, shouldn't happen because by that time
                            // Checker should've sent shutdown... 
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
    
    fn get_line(&mut self, n: usize) -> Option<String> {
        self.source.get_line(n)
            .map(|line| {
                line.iter().fold(String::new(), |mut acc, &c| {
                    acc.push(c.clone());
                    acc
                })
            })
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
           self.get_line(line as usize).map(|line| {
                write!(self.stdout, "{}\r\n", line)
           }); 
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
