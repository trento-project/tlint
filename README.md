# TLint
Validation engine for Trento Checks DSL.

## Usage
```sh
$ tlint -h
tlint 0.9.4

USAGE:
    tlint <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    lint
    show

```

TLint accepts standard input (until EOF):

```sh
$ cat check.yml | target/debug/tlint lint
  156F64   - expectations - List must not be empty
```

Or you can use the `-f` option to directly let TLint pick a file.

```sh
$ tlint lint -f check.yml
  156F64   - expectations - List must not be empty
```

## Running TLint over Docker
Currently if you don't want to build TLint yourself the most convenient solution is to run TLint over Docker.

You can put this useful alias into your shell configuration:

```sh
alias tlint='docker run --rm -i -v ${PWD}:/data ghcr.io/trento-project/tlint:latest'
```
