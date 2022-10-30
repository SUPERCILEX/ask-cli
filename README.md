# Ask CLI

`ask` offers a simple way to ask a yes or no question on the CLI, returning exit code 0 on "yes" and 1 on "no".

## Installation

### Use prebuilt binaries

Binaries for a number of platforms are available on the
[release page](https://github.com/SUPERCILEX/ask-cli/releases/latest).

### Build from source

```console,ignore
$ cargo +nightly install ask-cli
```

> To install cargo, follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Usage

Ask the user a question:

```bash
$ ask Do you want to continue?
Do you want to continue? [Y/n] yes
$ echo $?
0

$ ask Do you want to continue?
Do you want to continue? [Y/n] n
$ echo $?
1
```
