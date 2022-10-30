#![feature(read_buf)]

use std::{
    env, io,
    io::{BufRead, Write},
    process::ExitCode,
};

fn main() -> ExitCode {
    let question = question();
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();

    let mut reply = String::new();

    loop {
        stdout.write_all(question.as_bytes()).unwrap();
        stdout.write_all(b"[Y/n] ").unwrap();
        stdout.flush().unwrap();

        if stdin.read_line(&mut reply).unwrap() == 0 {
            return ExitCode::FAILURE;
        }
        reply.pop().unwrap();
        reply.make_ascii_lowercase();
        match reply.as_str() {
            "" | "y" | "yes" => return ExitCode::SUCCESS,
            "n" | "no" => return ExitCode::FAILURE,
            _ => {
                reply.clear();
            }
        }
    }
}

fn question() -> String {
    let words = || env::args().skip(1);

    let mut question = {
        let mut cap = 0;
        for word in words() {
            // + 1 for spaces
            cap += word.len() + 1;
        }
        String::with_capacity(cap)
    };

    for word in words() {
        question.push_str(&word);
        question.push(' ');
    }

    question
}
