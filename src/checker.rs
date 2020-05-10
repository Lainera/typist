use crate::{Parsed, Source};
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::sync::Arc;

pub(crate) enum Control {
    Previous(Option<char>, (usize, usize)),
    Next(Option<Result<char, char>>, (usize, usize)),
    Stop,
}

struct Cursor {
    source: Arc<Source>,
    cursor: (usize, usize),
}

impl Cursor {
    fn new(source: Arc<Source>) -> Self {
        Cursor {
            cursor: (0, 0),
            source,
        }
    }

    fn next(&mut self) -> (Option<char>, Option<(usize, usize)>) {
        let (row, column) = self.cursor;
        let symbol = self.source.get_char(row, column); 
        if let Some(&symbol) = symbol {
            self.cursor = (row, column + 1);
            return (Some(symbol), Some(self.cursor));
        }
        // No characters left in current line
        if self.source.get_line(row + 1).is_some() {
            self.cursor = (row + 1, 0);
            return (None, Some(self.cursor));
        }
        // End of the input
        return (None, None);
    }

    fn prev(&mut self) -> (Option<char>, Option<(usize, usize)>) {
        let (row, column) = self.cursor;
        // There are still preceding characters
        if column > 0 {
            let &symbol = self.source.get_char(row, column - 1).expect("Checker cursor desync");
            self.cursor = (row, column - 1);
            return (Some(symbol), Some(self.cursor));
        }
        // There are still preceding lines
        if row > 0 {
            let line = self.source
                .get_line(row - 1)
                .expect("Overflow on the lines previous");
            self.cursor = (row - 1, line.len());
            // Moving back one line leaves us on the spot that was occupied by `\n` or `\r\n`
            // Thus we return None
            return (None, Some(self.cursor));
        }
        // We are in the beginning of source
        (None, None)
    }
}

pub(crate) struct Checker {
    cursor: Cursor, 
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: SyncSender<()>,
}

impl Checker {
    pub(crate) fn new(
        input: Receiver<Parsed>,
        output: Sender<Control>,
        done: SyncSender<()>,
        source: Arc<Source>,
    ) -> Self {
        let cursor = Cursor::new(source);
        Self {
            done,
            input,
            output,
            cursor,
        }
    }
    
    pub(crate) fn run(self) -> Result<(), errors::CheckerError> {
        let mut source_cursor = self.cursor;
        for parsed in self.input {
            match parsed {
                Parsed::Stop => {
                    self.output.send(Control::Stop)?;
                    break;
                }
                Parsed::Backspace => {
                    match source_cursor.prev() {
                        (Some(source_symbol), Some(cursor)) => self
                            .output
                            .send(Control::Previous(Some(source_symbol), cursor))?,
                        (None, Some(cursor)) => {
                            self.output.send(Control::Previous(None, cursor))?
                        }
                        // Beginning of the string, backspacing does nothing
                        _ => {}
                    }
                }
                Parsed::Symbol(symbol) => match source_cursor.next() {
                    (Some(source_symbol), Some(cursor)) => {
                        if source_symbol == symbol {
                            self.output
                                .send(Control::Next(Some(Ok(source_symbol)), cursor))?
                        } else {
                            self.output
                                .send(Control::Next(Some(Err(source_symbol)), cursor))?
                        }
                    }
                    (None, Some(cursor)) => {
                        if symbol == '\n' {
                            self.output.send(Control::Next(None, cursor))?
                        } else {
                            source_cursor.prev();
                        }
                    }
                    (None, None) => {
                        self.done.send(())?;
                        self.output.send(Control::Stop)?;
                        break;
                    }
                    // End of the line, but symbol is not newline -> do nothing.
                    _ => {}
                },
            }
        }
        Ok(())
    }
}

mod errors {
    use crate::Control;
    use std::sync::mpsc::SendError;

    #[derive(Debug)]
    pub(crate) enum CheckerError {
        Control(SendError<Control>),
        Done(SendError<()>),
    }

    impl std::fmt::Display for CheckerError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Checker error \n {}", self)
        }
    }

    impl std::error::Error for CheckerError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                CheckerError::Control(error) => Some(error),
                CheckerError::Done(error) => Some(error),
            }
        }
    }

    impl From<SendError<Control>> for CheckerError {
        fn from(error: SendError<Control>) -> Self {
            CheckerError::Control(error)
        }
    }

    impl From<SendError<()>> for CheckerError {
        fn from(error: SendError<()>) -> Self {
            CheckerError::Done(error)
        }
    }
}
