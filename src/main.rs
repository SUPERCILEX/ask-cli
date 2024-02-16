use std::{env, ffi::OsString, process::Termination};

use ask_cli::{ask, Answer};

fn main() -> impl Termination {
    let mut question = OsString::new();
    parse_question(&mut question);

    // TODO https://github.com/rust-lang/libs-team/issues/148
    #[cfg(unix)]
    {
        use std::{
            fs::File,
            mem::ManuallyDrop,
            os::unix::{ffi::OsStrExt, io::FromRawFd},
        };

        let mut stdin = ManuallyDrop::new(unsafe { File::from_raw_fd(0) });
        let mut stdout = ManuallyDrop::new(unsafe { File::from_raw_fd(1) });
        ask(question.as_bytes(), Answer::Yes, &mut *stdin, &mut *stdout)
    }
    #[cfg(not(unix))]
    {
        use std::io;

        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        ask(
            question.to_string_lossy().as_bytes(),
            Answer::Yes,
            &mut stdin,
            &mut stdout,
        )
    }
}

fn parse_question(question: &mut OsString) {
    let words = env::args_os().skip(1);

    for word in words {
        question.reserve(word.len() + 1);

        question.push(&word);
        question.push(" ");
    }
    question.push("[Y/n] ");
}
