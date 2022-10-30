#![feature(read_buf)]

use std::{
    env, io,
    io::{BorrowedBuf, Read, StdinLock, Write},
    mem,
    mem::MaybeUninit,
    process::ExitCode,
    str::from_utf8,
};

fn main() -> ExitCode {
    let question = question();
    let mut stdout = io::stdout().lock();
    let mut stdin = io::stdin().lock();

    // max_len(yes, no, y, n) = 3 -> 3 + 2 bytes for new lines
    let (mut buf, mut buf2) = ([MaybeUninit::uninit(); 5], [MaybeUninit::uninit(); 5]);
    let (mut buf, mut buf2) = (
        BorrowedBuf::from(buf.as_mut()),
        BorrowedBuf::from(buf2.as_mut()),
    );

    'outer: loop {
        stdout.write_all(question.as_bytes()).unwrap();
        stdout.write_all(b"[Y/n] ").unwrap();
        stdout.flush().unwrap();

        let newline_index = if let Some(newline_index) = read_newline_index(&mut stdin, &mut buf) {
            newline_index
        } else {
            loop {
                buf.clear();

                if let Some(newline_index) = read_newline_index(&mut stdin, &mut buf) {
                    consume_newline(&mut buf, &mut buf2, newline_index);
                    mem::swap(&mut buf, &mut buf2);
                    continue 'outer;
                }

                if buf.len() == 0 {
                    // Reached EOF
                    return ExitCode::FAILURE;
                }
            }
        };

        let reply = from_utf8(&buf.filled()[..newline_index]).unwrap();
        // TODO https://github.com/rust-lang/rust/pull/103754
        match reply.to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return ExitCode::SUCCESS,
            "n" | "no" => return ExitCode::FAILURE,
            _ => {
                consume_newline(&mut buf, &mut buf2, newline_index);
                mem::swap(&mut buf, &mut buf2);
            }
        }
    }
}

fn consume_newline(buf: &mut BorrowedBuf, buf2: &mut BorrowedBuf, newline_index: usize) {
    buf2.clear();
    buf2.unfilled().append(&buf.filled()[newline_index + 1..]);
}

fn read_newline_index(stdin: &mut StdinLock, buf: &mut BorrowedBuf) -> Option<usize> {
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
