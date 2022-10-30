#![feature(read_buf)]

use std::{
    env, io,
    io::{BorrowedBuf, Read, Write},
    mem,
    mem::MaybeUninit,
    process::ExitCode,
    str::from_utf8,
};

fn main() -> ExitCode {
    // max_len(yes, no, y, n) = 3 -> 3 + 2 bytes for new lines
    const BUF_LEN: usize = 5;

    let (mut buf, mut buf2) = (
        [MaybeUninit::uninit(); BUF_LEN],
        [MaybeUninit::uninit(); BUF_LEN],
    );
    let (mut buf, mut buf2) = (
        BorrowedBuf::from(buf.as_mut()),
        BorrowedBuf::from(buf2.as_mut()),
    );

    macro_rules! consume_bytes {
        ($count:expr) => {
            buf2.clear();
            buf2.unfilled().append(&buf.filled()[$count..]);
            mem::swap(&mut buf, &mut buf2);
        };
    }

    macro_rules! consume_newline {
        ($newline_index:expr) => {
            let newline_index = $newline_index;
            let is_crlf =
                buf.filled()[newline_index] == b'\r' && buf.filled()[newline_index + 1] == b'\n';
            let skip = if is_crlf { 2 } else { 1 };
            consume_bytes!(newline_index + skip);
        };
    }

    let mut stdin = io::stdin().lock();

    /// Continuously reads from stdin until encountering a newline, returning
    /// the index of its first byte.
    macro_rules! read_line {
        () => {{
            let mut failed = false;
            loop {
                stdin.read_buf(buf.unfilled()).unwrap();
                if buf.len() == 0 {
                    // Reached EOF
                    return ExitCode::from(2);
                }

                match buf.filled().iter().position(|b| *b == b'\n' || *b == b'\r') {
                    Some(newline_index) if newline_index == BUF_LEN - 1 => {
                        failed = true;
                        // Potentially read in a '\n' to make a CRLF
                        consume_bytes!(1);
                    }
                    Some(newline_index) => {
                        break if failed {
                            consume_newline!(newline_index);
                            None
                        } else {
                            Some(newline_index)
                        }
                    }
                    None => {
                        if buf.len() == buf.capacity() {
                            failed = true;
                            buf.clear();
                        }
                    }
                }
            }
        }};
    }

    let question = question();
    let mut stdout = io::stdout().lock();

    loop {
        stdout.write_all(question.as_bytes()).unwrap();
        stdout.write_all(b"[Y/n] ").unwrap();
        stdout.flush().unwrap();

        let newline_index = if let Some(newline_index) = read_line!() {
            newline_index
        } else {
            continue;
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
