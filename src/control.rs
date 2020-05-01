pub(crate) enum Control {
    Backspace,
    Enter,
    Stop,
    Symbol(Result<char, char>),
}
