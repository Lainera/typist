use std::io;
use std::io::prelude::*;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let word = String::from("Hello");
        let size = word.len();
        let letters = word.chars();
        let duplex = rx.into_iter().zip(letters);
        let mut errors = 0;
        let mut counter = 0;
        for (a, b) in duplex.peekable() {
            println!("Received:{}, Actual:{}", a, b);
            if a != b { errors += 1}
            counter += 1;
            if counter == size {
                break;
            }
        }
        println!("Error count: {}", errors);
        std::process::exit(0)
    });

    for line in io::stdin().lock().lines() {
        for letter in line.unwrap().chars() {
            tx.send(letter)
                .expect("Failed to send letter");
        }
    }
}
