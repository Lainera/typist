use crate::{source::Source, Parsed};
use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::sync::Arc;

pub(crate) enum Control {
    Previous(Option<char>, (usize, usize)),
    Next(Option<Result<char, char>>, (usize, usize)),
    Stop,
}

struct Cursor<S: Source> {
    source: Arc<S>,
    cursor: (usize, usize),
}

impl <S:Source>Cursor<S> {
    fn new(source: Arc<S>) -> Self {
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
            let &symbol = self
                .source
                .get_char(row, column - 1)
                .expect("Checker cursor desync");
            self.cursor = (row, column - 1);
            return (Some(symbol), Some(self.cursor));
        }
        // There are still preceding lines
        if row > 0 {
            let line = self
                .source
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

pub(crate) struct Checker<S: Source> {
    cursor: Cursor<S>,
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: SyncSender<()>,
}

impl <S: Source>Checker<S> {
    pub(crate) fn new(
        input: Receiver<Parsed>,
        output: Sender<Control>,
        done: SyncSender<()>,
        source: Arc<S>,
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
        let mut cursor = self.cursor;
        for parsed in self.input {
            match parsed {
                Parsed::Stop => {
                    self.output.send(Control::Stop)?;
                    break;
                }
                Parsed::Backspace => {
                    match cursor.prev() {
                        (Some(source_symbol), Some((row, column))) => self
                            .output
                            .send(Control::Previous(Some(source_symbol), (row, column)))?,
                        (None, Some((row, column))) => {
                            self.output.send(Control::Previous(None, (row, column)))?
                        }
                        // Beginning of the string, backspacing does nothing
                        _ => {}
                    }
                }
                Parsed::Symbol(symbol) => match cursor.next() {
                    (Some(source_symbol), Some((row, column))) => {
                        if source_symbol == symbol {
                            self.output
                                .send(Control::Next(Some(Ok(source_symbol)), (row, column)))?
                        } else {
                            self.output
                                .send(Control::Next(Some(Err(source_symbol)), (row, column)))?
                        }
                    }
                    (None, Some((row, column))) => {
                        if symbol == '\n' {
                            self.output.send(Control::Next(None, (row, column)))?
                        } else {
                            cursor.prev();
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
