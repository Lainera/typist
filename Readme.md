## Muscle memory is a thing.

---
## Install

- [install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- `git clone`
- `cargo install --path .`

## Practice
- `typist %SOURCE_FILE`

## Project Structure 

- Parser: processes keyboard input, sends downstream to Checker.
- Checker: processes input from Parser, traverses source string, compares input with source. When done tells Parser to stop. Until done, sends downstream outcome of input to Renderer. 
- Renderer: processes input from Checker, draws on terminal.

