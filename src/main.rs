#![feature(read_buf)]

use std::{
    borrow::Cow,
    env,
    ffi::OsString,
    io,
    io::{BorrowedBuf, Read, Write},
    mem,
    mem::MaybeUninit,
    process::ExitCode,
};

#[allow(clippy::too_many_lines)]
fn main() -> ExitCode {
    // max_len(yes, no, y, n) = 3 -> 3 + 2 bytes for new lines
    const BUF_LEN: usize = 5;

    enum State {
        Start,
        Ask {
            /// Passthrough for [State::Read].
            pending_crlf: bool,
        },
        /// Continuously reads from stdin until encountering a newline,
        /// returning the index of its first byte.
        ///
        /// This state deals with a number of edge cases:
        /// - If stdin reaches the EOF, exit the process *after* processing all
        ///   remaining input.
        /// - If a \r byte is seen on the border of the buffer, fail the input
        ///   (because it will be at a higher index than the max length of all
        ///   possible valid replies and therefore cannot be a valid reply) and
        ///   consume a \n if it is the next byte. Note that this consumption
        ///   cannot happen before printing the question again or we might get
        ///   blocked on stdin if there are no more bytes available.
        /// - If no newline was found within the buffer, fail the reply since it
        ///   cannot possibly be valid.
        Read {
            /// The reply is known to be invalid, but we have not yet seen a
            /// newline.
            failed: bool,
            /// A CRLF might be striding the buffer.
            pending_crlf: bool,
        },
        HandleReply {
            newline_index: usize,
        },
    }

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
            let is_crlf = buf.filled()[newline_index] == b'\r'
                && matches!(buf.filled().get(newline_index + 1), Some(b'\n'));
            let skip = if is_crlf { 2 } else { 1 };
            consume_bytes!(newline_index + skip);
        };
    }

    let mut question = OsString::new();
    let question = parse_question(&mut question);

    let mut stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut state = State::Start;
    loop {
        state = match state {
            State::Start => State::Ask {
                pending_crlf: false,
            },
            State::Ask { pending_crlf } => {
                stdout.write_all(question.as_bytes()).unwrap();
                stdout.write_all(b"[Y/n] ").unwrap();
                stdout.flush().unwrap();

                State::Read {
                    failed: false,
                    pending_crlf,
                }
            }
            State::Read {
                failed,
                pending_crlf,
            } => {
                debug_assert!(buf.len() < buf.capacity());

                stdin.read_buf(buf.unfilled()).unwrap();
                if buf.len() == 0 {
                    // Reached EOF
                    return ExitCode::from(2);
                }

                if pending_crlf && buf.filled()[0] == b'\n' {
                    consume_bytes!(1);
                }

                match buf.filled().iter().position(|b| *b == b'\n' || *b == b'\r') {
                    Some(newline_index) if newline_index == BUF_LEN - 1 => {
                        let pending_crlf = buf.filled()[newline_index] == b'\r';
                        buf.clear();
                        State::Ask { pending_crlf }
                    }
                    Some(newline_index) if failed => {
                        consume_newline!(newline_index);
                        State::Ask {
                            pending_crlf: false,
                        }
                    }
                    Some(newline_index) => State::HandleReply { newline_index },
                    None if buf.len() == buf.capacity() => {
                        buf.clear();
                        State::Read {
                            failed: true,
                            pending_crlf: false,
                        }
                    }
                    None => State::Read {
                        failed: false,
                        pending_crlf: false,
                    },
                }
            }
            State::HandleReply { newline_index } => {
                let reply = &buf.filled()[..newline_index];
                // TODO https://github.com/rust-lang/rust/pull/103754
                match reply.to_ascii_lowercase().as_slice() {
                    b"" | b"y" | b"yes" => return ExitCode::SUCCESS,
                    b"n" | b"no" => return ExitCode::FAILURE,
                    _ => {
                        consume_newline!(newline_index);
                        State::Ask {
                            pending_crlf: false,
                        }
                    }
                }
            }
        }
    }
}

fn parse_question(question: &mut OsString) -> Cow<'_, str> {
    let words = || env::args_os().skip(1);

    question.reserve(words().len() + words().map(|word| word.len()).sum::<usize>());
    for word in words() {
        question.push(&word);
        question.push(" ");
    }
    question.to_string_lossy()
}
