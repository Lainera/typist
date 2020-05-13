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
    cursor: Cursor, 
}

struct Cursor {
    head: usize,
    tail: usize,
    window_size: u16,
    source: Arc<Source>,
}

impl Cursor {
    fn new(source: Arc<Source>) -> Result<Self, io::Error> {
        let (_, window_size) = termion::terminal_size()?;
        Ok(Self {
            head: 0,
            tail: 0,
            source,
            window_size: window_size - 1,
        })
    }
    
    fn can_scroll_up(&self) -> bool {
        self.head > 0 && 
        self.tail > 0 && 
        absolute_difference(self.head, self.tail) == 0
    }

    fn is_at_the_bottom_of_the_screen(&self) -> bool {
        absolute_difference(self.head + 1, self.tail) as u16 >= self.window_size
    }
    
    // Need to be able to move head within terminal window 
    fn move_head_up(&mut self) {
        if self.head.checked_sub(1).is_some() {
            self.head -= 1;
        };
    }

    // As well as be able to move window itself.
    fn scroll_up(&mut self) {
        match (self.head.checked_sub(1), self.tail.checked_sub(1)) {
            (Some(adjusted_head), Some(adjusted_tail)) => {
                self.head = adjusted_head;
                self.tail = adjusted_tail; 
            },
            _ => ()
        }
    }
    
    fn move_head_down(&mut self) {
        self.head += 1;
    }
    
    fn scroll_down(&mut self) {
        self.head += 1;
        self.tail += 1;
    }

    fn get_line(&self, n: usize) -> Option<String> {
        self.source.get_line(n)
            .map(|line| {
                line.iter().fold(String::new(), |mut acc, &c| {
                    acc.push(c.clone());
                    acc
                })
            })
    }

    fn get_bottom_line(&self) -> Option<String> {
        self.get_line(self.head)
    }

    fn get_top_line(&self) -> Option<String> {
        self.get_line(self.tail)
    }
    
    // Tail represents top of the screen, thus actual
    // position of cursor on the screen is row - tail
    fn adjust_row(&self, row: usize) -> usize {
        match row.checked_sub(self.tail) {
            Some(adjusted) => adjusted,
            None => row,
        }
    }
}

fn absolute_difference(a: usize, b: usize) -> u32 {
   let i = a as i32 - b as i32;
    if i < 0 {
        -i as u32
    } else {
        i as u32
    }
}

// ANSI terminals are 1 based
fn ansi_goto(row: usize, column: usize) -> termion::cursor::Goto {
   termion::cursor::Goto((column + 1) as u16, (row + 1) as u16) 
}

impl Renderer {
    pub(crate) fn new(stdout: Stdout, input: Receiver<Control>, source: Arc<Source>) -> Result<Self, io::Error> {
        let stdout = stdout.into_raw_mode()?;
        let cursor = Cursor::new(source)?;
        Ok(Self {
            stdout,
            input,
            cursor,
        })
    }

    pub(crate) fn run(mut self) -> Result<(), io::Error> {
        self.draw_initial()?;
        let mut cursor = self.cursor;
        for c in self.input {
            match c {
                Control::Stop => {
                    // Restore everything back to normal on Stop command;
                    write!(self.stdout, "{}", termion::cursor::Restore)?;
                    self.stdout.flush()?;
                    break;
                }
                // Backspacing within same line.
                Control::Previous(Some(symbol), _) => write!(
                    self.stdout,
                    "{}{}{}",
                    termion::cursor::Left(1),
                    symbol,
                    termion::cursor::Left(1)
                )?,
                // At the beginning of the line and need to move cursor up
                Control::Previous(None, (row, column)) => if cursor.can_scroll_up() {
                    cursor.scroll_up();
                    let line = cursor.get_top_line().expect("Render cursor is not aligned");
                    write!(
                        self.stdout,
                        "{}{}{}{}",
                        // cursor scrolls up, but terminal 
                        // actually scrolls down to make room for newline
                        termion::scroll::Down(1),
                        ansi_goto(0, 0),
                        line,
                        ansi_goto(0, column),
                    )?
                } else {
                    cursor.move_head_up(); 
                    write!(
                        self.stdout,
                        "{}",
                        ansi_goto(cursor.adjust_row(row), column),
                    )?;
                },
                // At the end of the line
                Control::Next(None, (row, column)) => if cursor.is_at_the_bottom_of_the_screen() {
                    cursor.scroll_down();
                    if let Some(line) = cursor.get_bottom_line() {
                        write!(
                            self.stdout,
                            "{}\r{}{}",
                            termion::scroll::Up(1),
                            line,
                            ansi_goto(cursor.adjust_row(row), column),
                        )?
                    } 
                } else {
                    cursor.move_head_down();
                    write!(
                        self.stdout,
                        "{}",
                        ansi_goto(cursor.adjust_row(row), column),
                    )?;
                },
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

    pub(crate) fn draw_initial(&mut self) -> Result<(), io::Error> {
        write!(
            self.stdout,
            "{}{}{}",
            termion::clear::All,
            ansi_goto(0, 0),
            termion::cursor::Hide
        )?;

        for line in 0..self.cursor.window_size {
           self.cursor.get_line(line as usize).map(|line| {
                write!(self.stdout, "{}\r\n", line)
           }); 
        }

        write!(
            self.stdout,
            "{}{}{}",
            ansi_goto(0, 0),
            termion::cursor::Show,
            termion::cursor::BlinkingUnderline
        )?;

        self.stdout.flush()
    }
}
