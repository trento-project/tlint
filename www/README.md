# TLint web
TLint web runs `tlint` checks on a web application running `tlint` as web assembly code.

## Development
Use `rustup` to install and configure `rust`.
Compiled version of `rust` distributed in SUSE repositories doesn't match with downloaded version of `wasm`.

Get `wasm-pack` [here](https://rustwasm.github.io/wasm-pack/installer/).

```
wasm-pack build
npm install
npm run build
npm run serve
```

## Docker
Build TLint web docker image running:

```
docker build . -t tlint-web -f Dockerfile.www
```