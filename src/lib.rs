#![feature(read_buf)]

use std::{
    io,
    io::{BorrowedBuf, Read, Write},
    mem,
    mem::MaybeUninit,
    process::{ExitCode, Termination},
};

/// The answer to a question posed in [ask].
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Answer {
    Yes,
    No,
    Unknown,
}

impl Termination for Answer {
    fn report(self) -> ExitCode {
        match self {
            Self::Yes => ExitCode::SUCCESS,
            Self::No => ExitCode::FAILURE,
            Self::Unknown => ExitCode::from(2),
        }
    }
}

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
    ///   consume a \n if it is the next byte. Note that this consumption cannot
    ///   happen before printing the question again or we might get blocked on
    ///   stdin if there are no more bytes available.
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

/// Ask the user a yes or no question on stdout, reading the reply from stdin.
///
/// Replies are delimited by newlines of any kind and must be one of '' (maps to
/// yes), 'y', 'yes', 'n', 'no', case insensitive. If the reply fails to parse,
/// the question will be asked again ad infinitum.
///
/// # Examples
///
/// ```
/// # use std::{io, io::Read, str::from_utf8};
/// use ask_cli::{ask, Answer};
///
/// assert!(matches!(
///     ask("Continue? [Y/n] ", &mut "y\n".as_bytes(), &mut io::sink()),
///     Ok(Answer::Yes)
/// ));
/// assert!(matches!(
///     ask("Continue? [Y/n] ", &mut "n\n".as_bytes(), &mut io::sink()),
///     Ok(Answer::No)
/// ));
/// assert!(matches!(
///     ask("Continue? [Y/n] ", &mut "".as_bytes(), &mut io::sink()),
///     Ok(Answer::Unknown)
/// ));
///
/// // Here we use 3 different kinds of line endings
/// let mut stdout = Vec::new();
/// ask(
///     "Continue? [Y/n] ",
///     &mut "a\nb\rc\r\nyes\n".as_bytes(),
///     &mut stdout,
/// )
/// .unwrap();
/// assert_eq!(
///     "Continue? [Y/n] Continue? [Y/n] Continue? [Y/n] Continue? [Y/n] ",
///     from_utf8(&stdout).unwrap()
/// );
/// ```
///
/// # Errors
///
/// Underlying I/O errors are bubbled up.
pub fn ask<Q: AsRef<str>, In: Read, Out: Write>(
    question: Q,
    stdin: &mut In,
    stdout: &mut Out,
) -> Result<Answer, io::Error> {
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
            let is_crlf = buf.filled()[newline_index] == b'\r'
                && matches!(buf.filled().get(newline_index + 1), Some(b'\n'));
            let skip = if is_crlf { 2 } else { 1 };
            consume_bytes!(newline_index + skip);
        };
    }

    let mut state = State::Start;
    loop {
        state = match state {
            State::Start => State::Ask {
                pending_crlf: false,
            },
            State::Ask { pending_crlf } => {
                stdout.write_all(question.as_ref().as_bytes())?;
                stdout.flush()?;
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

                let prev_count = buf.len();
                stdin.read_buf(buf.unfilled())?;

                if pending_crlf && matches!(buf.filled().first(), Some(b'\n')) {
                    consume_bytes!(1);
                }

                match buf.filled().iter().position(|&b| b == b'\n' || b == b'\r') {
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
                    None if !pending_crlf && buf.len() == prev_count => {
                        // Reached EOF
                        return Ok(Answer::Unknown);
                    }
                    None => State::Read {
                        failed,
                        pending_crlf: false,
                    },
                }
            }
            State::HandleReply { newline_index } => {
                let reply = &buf.filled()[..newline_index];
                // TODO https://github.com/rust-lang/rust/pull/103754
                match reply.to_ascii_lowercase().as_slice() {
                    b"" | b"y" | b"yes" => return Ok(Answer::Yes),
                    b"n" | b"no" => return Ok(Answer::No),
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

#[cfg(kani)]
#[kani::proof]
#[kani::unwind(9)]
fn ask_proof() {
    let input: [u8; 4] = kani::any();
    let output = ask("?", &mut input.as_slice(), &mut io::sink());

    output.unwrap();
}
