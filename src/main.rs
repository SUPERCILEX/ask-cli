#![feature(read_buf)]

use std::{
    env, io,
    io::{BorrowedBuf, Read, StdinLock, Write},
    mem::MaybeUninit,
    process::ExitCode,
    str::from_utf8,
};

fn main() -> ExitCode {
    let question = question();
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();

    // max_len(yes, no, y, n) = 3 -> 3 + 2 bytes for new lines
    let mut buf = [MaybeUninit::uninit(); 5];
    let mut buf = BorrowedBuf::from(buf.as_mut());

    loop {
        stdout.write_all(question.as_bytes()).unwrap();
        stdout.write_all(b"[Y/n] ").unwrap();
        stdout.flush().unwrap();

        let reply = from_utf8({
            if let Some(newline_index) = newline_index(&mut stdin, &mut buf) {
                &buf.filled()[..newline_index]
            } else {
                while newline_index(&mut stdin, &mut buf).is_none() {}
                continue;
            }
        })
        .unwrap();
        // TODO https://github.com/rust-lang/rust/pull/103754
        match reply.to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return ExitCode::SUCCESS,
            "n" | "no" => return ExitCode::FAILURE,
            _ => {}
        }
    }
}

fn newline_index(stdin: &mut StdinLock, buf: &mut BorrowedBuf) -> Option<usize> {
    buf.clear();
    stdin.read_buf(buf.unfilled()).unwrap();
    buf.filled().iter().position(|b| *b == b'\n' || *b == b'\r')
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
