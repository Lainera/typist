use crate::Parsed;
use std::sync::mpsc::{Sender, Receiver};

pub(crate) enum Control {
    Backspace(char),
    PreviousLine,
    Enter,
    Stop,
    Symbol(Result<char, char>),
}

struct StringCursor {
    source: Vec<char>,
    cursor: usize,
}

impl StringCursor {
    fn new<S: Into<String>>(source: S) -> Self {
        let source = source
            .into()
            .chars()
            .collect::<Vec<char>>();

        Self {
            source,
            cursor: 0
        }
    }
    // Need to include some more info here. Potentially line number and length. 
    fn next(&mut self) -> Option<&char> {
        let symbol = self.source.get(self.cursor);
        if symbol.is_some() {
            self.cursor += 1;
        }
        symbol
    }
    
    fn prev(&mut self) -> Option<&char> {
        // Beginning of source? -> Nowhere to run
        if self.cursor.checked_sub(1).is_none() {
            None
        } else {
        // Otherwise rewind cursor one position and spit out contents. 
            self.cursor -= 1;
            self.source.get(self.cursor)
        }
    }
}

pub(crate) struct Checker {
    source: StringCursor,
    input: Receiver<Parsed>,
    output: Sender<Control>,
    done: Sender<Control>,
}

impl Checker {
    pub(crate) fn new(
        input: Receiver<Parsed>, 
        output: Sender<Control>, 
        done: Sender<Control>, 
        source_string: String
    ) -> Self {
        let source = StringCursor::new(source_string); 
        Self {
            done,
            input,
            output,
            source,
        }
    }

    pub(crate) fn run(self) -> Result<(), std::sync::mpsc::SendError<Control>> {
        let mut source = self.source;
        for parsed in self.input {
            match parsed {
                Parsed::Stop => {
                    self.output.send(Control::Stop)?;
                    break;
                },
                Parsed::Backspace => {
                    match source.prev() {
                        Some(&source_symbol) => {
                            // Maybe need to handle variety of other newline chars?
                            if source_symbol == '\n' {
                                self.output.send(Control::PreviousLine)?
                            } else {
                                self.output.send(Control::Backspace(source_symbol))?
                            }
                        },
                        // We are at first symbol in source string
                        None => (),
                    }
                },
                Parsed::Symbol(symbol) => {
                   match source.next() {
                       Some(&source_symbol) if source_symbol == '\n' => {
                           if source_symbol == symbol {
                               self.output.send(Control::Enter)?
                           } else {
                               source.prev();
                           }
                       },
                       Some(&source_symbol) => {
                           if source_symbol == symbol {
                               self.output.send(Control::Symbol(Ok(symbol)))?
                           } else {
                               self.output.send(Control::Symbol(Err(symbol)))?
                           }
                       },
                       None => {
                           self.done.send(Control::Stop)?;
                           self.output.send(Control::Stop)?;
                           break;
                       }
                   }
                },
            }
        }
        Ok(())
    }
}

