pub(crate) enum Control {
    Backspace(char),
    PreviousLine,
    Enter,
    Stop,
    Symbol(Result<char, char>),
}
