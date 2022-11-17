# Rashomon
Validation engine for Trento Checks DSL.

## Usage
```sh
$ tlint -h
tlint 0.9.0

USAGE:
    tlint [OPTIONS]

OPTIONS:
    -f, --file <FILE>    
    -h, --help           Print help information
    -V, --version        Print version information
```

Rashomon accepts standard input (until EOF):

```sh
$ cat check.yml | target/debug/tlint
  156F64   - expectations - List must not be empty
```

Or you can use the `-f` option to directly let Rashomon pick a file.

```sh
$ tlint -f check.yml 
  156F64   - expectations - List must not be empty
```
