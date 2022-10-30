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
    let (mut buf, mut buf2) = ([MaybeUninit::uninit(); 6], [MaybeUninit::uninit(); 6]);
    let (mut buf, mut buf2) = (
        BorrowedBuf::from(buf.as_mut()),
        BorrowedBuf::from(buf2.as_mut()),
    );

    // TODO docs
    macro_rules! consume_newline {
        ($newline_index:expr) => {
            let newline_index = $newline_index;
            let next_index = if buf.filled()[newline_index] == b'\r' {
                match buf.filled().get(newline_index + 1) {
                    Some(c) if *c == b'\n' => newline_index + 2,
                    Some(_) => newline_index + 1,
                    None => newline_index - 1,
                }
            } else {
                newline_index + 1
            };

            buf2.clear();
            buf2.unfilled().append(&buf.filled()[next_index..]);
            mem::swap(&mut buf, &mut buf2);
        };
    }

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
                    consume_newline!(newline_index);
                    continue 'outer;
                }

                if buf.len() == 0 {
                    // Reached EOF
                    return ExitCode::from(2);
                }
            }
        };

        let reply = from_utf8(&buf.filled()[..newline_index]).unwrap();
        // TODO https://github.com/rust-lang/rust/pull/103754
        match reply.to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return ExitCode::SUCCESS,
            "n" | "no" => return ExitCode::FAILURE,
            _ => {
                consume_newline!(newline_index);
            }
        }
    }
}

fn read_newline_index(stdin: &mut StdinLock, buf: &mut BorrowedBuf) -> Option<usize> {
    stdin.read_buf(buf.unfilled()).unwrap();
    buf.filled().iter().position(|b| *b == b'\n' || *b == b'\r')
}

fn question() -> String {
    let words = || env::args().skip(1);

    let mut question =
        String::with_capacity(words().len() + words().map(|word| word.len()).sum::<usize>());
    for word in words() {
        question.push_str(&word);
        question.push(' ');
    }
    question
}
