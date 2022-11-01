use ask_cli::ask;
use std::{borrow::Cow, env, ffi::OsString, process::Termination};

fn main() -> impl Termination {
    let mut question = OsString::new();
    let question = parse_question(&mut question);

    #[cfg(unix)]
    {
        use std::{fs::File, mem::ManuallyDrop, os::fd::FromRawFd};

        let mut stdin = ManuallyDrop::new(unsafe { File::from_raw_fd(0) });
        let mut stdout = ManuallyDrop::new(unsafe { File::from_raw_fd(1) });
        ask(question, &mut *stdin, &mut *stdout).unwrap()
    }
    #[cfg(not(unix))]
    {
        use std::io;

        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();
        ask(&question, &mut stdin, &mut stdout).unwrap()
    }
}

fn parse_question(question: &mut OsString) -> Cow<'_, str> {
    let words = env::args_os().skip(1);

    for word in words {
        question.reserve(word.len() + 1);

        question.push(&word);
        question.push(" ");
    }
    question.push("[Y/n] ");

    question.to_string_lossy()
}
