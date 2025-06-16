# TLint

Validation engine for Trento Checks DSL.

## Usage

TLint is used to easily check whether [Trento Checks][checks] are valid and
up-to-date.

TLint accepts checks from standard input (until EOF):

```sh
$ cat check.yml | target/debug/tlint lint
  156F64   - expectations - List must not be empty
```

Furthermore, you can use a positional argument to read a check, directly.

```sh
$ tlint lint -f check.yml
  156F64   - expectations - List must not be empty
```

You can opt into or opt out of rules by using the `--rules` option. This can be
useful for skipping link validation. See `--help` for possible rules.

## Running TLint over Docker

Currently, if you don't want to build TLint yourself, the most convenient
solution is to run TLint over Docker.

To make this process more convenient, you can put this alias into your shell
configuration (~/.bashrc or equivalent):

```sh
alias tlint='docker run --rm -i -v ${PWD}:/data ghcr.io/trento-project/tlint:latest'
```

[checks]: https://github.com/trento-project/checks/
