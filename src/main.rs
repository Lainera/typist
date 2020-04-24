use std::io;
use std::io::prelude::*;
use std::sync::mpsc;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() -> Result<(), io::Error> {

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().expect("failed to convert to raw mode");
    let word = std::fs::read_to_string("./some-text.txt")?;
    write!(stdout, 
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::BlinkingUnderline
        ).unwrap();
    for (ix, line) in word.lines().enumerate() {
        write!(stdout, "{}{}", line, termion::cursor::Goto(1, (ix + 2) as u16)).unwrap();
    }
    stdout.flush().unwrap();
    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Ctrl(c) if c == 'c' => break,
            Key::Char('\n') => {
                write!(stdout, "\n{}", termion::cursor::Left(std::u16::MAX)).unwrap()
            },
            Key::Char(c) => {
                write!(stdout, "{}", c).unwrap()
            },
            _ => {},
        }
        stdout.flush().unwrap();
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();
//    let (tx, rx) = mpsc::channel();
//    thread::spawn(move || {
//        let word = String::from("Hello");
//        let size = word.len();
//        let letters = word.chars();
//        let duplex = rx.into_iter().zip(letters);
//        let mut errors = 0;
//        let mut counter = 0;
//        for (a, b) in duplex.peekable() {
//            println!("Received:{}, Actual:{}", a, b);
//            if a != b { errors += 1}
//            counter += 1;
//            if counter == size {
//                break;
//            }
//        }
//        println!("Error count: {}", errors);
//        std::process::exit(0)
//    });

//    for line in io::stdin().lock().lines() {
//        for letter in line.unwrap().chars() {
//            tx.send(letter)
//                .expect("Failed to send letter");
//        }
//    }
    Ok(())
}
