use crate::Parsed;
use std::sync::mpsc::{
    Receiver, 
    Sender, 
    SyncSender,
};

pub(crate) enum Control {
    Backspace(char),
    GoTo((usize, usize)),
    Symbol(Result<char, char>),
    Stop,
}

struct StringCursor {
    source: Vec<Vec<char>>,
    cursor: (usize, usize),
}

impl StringCursor {
    fn new<S: Into<String>>(source: S) -> Self {
        let source: Vec<Vec<char>> = source
            .into()
            .lines()
            .map(|line| line.chars().collect::<Vec<char>>())
            .collect();
        Self {
            source,
            cursor: (0, 0),
        }
    }

    fn next(&mut self) -> (Option<&char>, Option<(usize, usize)>) {
        let (row, column) = self.cursor;
        let line = self.source.get(row).expect("Overflow on the lines next");
        let symbol = line.get(column);
        if symbol.is_some() {
            self.cursor = (row, column + 1);
            return (symbol, Some(self.cursor));
        }
        // No characters left in current line
        if self.source.get(row + 1).is_some() {
            self.cursor = (row + 1, 0);
            return (None, Some(self.cursor));
        }
        // End of the input
        return (None, None);
    }

    fn prev(&mut self) -> (Option<&char>, Option<(usize, usize)>) {
        let (row, column) = self.cursor;
        // There are still preceding characters
        if column > 0 {
            let line = self
                .source
                .get(row)
                .expect("Overflow on the lines previous");
            self.cursor = (row, column - 1);
            return (line.get(column - 1), Some(self.cursor));
        }
        // There are still preceding lines
        if row > 0 {
            let line = self
                .source
                .get(row - 1)
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
    source: StringCursor,
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: SyncSender<()>,
}

impl Checker {
    pub(crate) fn new(
        input: Receiver<Parsed>,
        output: Sender<Control>,
        done: SyncSender<()>,
        source_string: String,
    ) -> Self {
        let source = StringCursor::new(source_string);
        Self {
            done,
            input,
            output,
            source,
        }
    }

    pub(crate) fn run(self) -> Result<(), errors::CheckerError> {
        let mut source = self.source;
        for parsed in self.input {
            match parsed {
                Parsed::Stop => {
                    self.output.send(Control::Stop)?;
                    break;
                }
                Parsed::Backspace => {
                    match source.prev() {
                        (Some(&source_symbol), Some(_)) => {
                            self.output.send(Control::Backspace(source_symbol))?
                        }
                        (None, Some(cursor)) => self.output.send(Control::GoTo(cursor))?,
                        // Beginning of the string, backspacing does nothing
                        _ => {}
                    }
                }
                Parsed::Symbol(symbol) => match source.next() {
                    (Some(&source_symbol), Some(_)) => {
                        if source_symbol == symbol {
                            self.output.send(Control::Symbol(Ok(source_symbol)))?
                        } else {
                            self.output.send(Control::Symbol(Err(source_symbol)))?
                        }
                    }
                    (None, Some(cursor)) => {
                        if symbol == '\n' {
                            self.output.send(Control::GoTo(cursor))?
                        } else {
                            source.prev();
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
