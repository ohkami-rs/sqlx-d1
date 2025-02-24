# Cloudflare Workers template

## Prerequisites

- Recent Rust toolchain of with `wasm32-unknown-unknown` target
- [`worker-build`](https://crates.io/crates/worker-build) ( run `cargo install worker-build` to install )
- npm

In addition, `wasm-opt` ( pakcaged in [binaryen](https://github.com/WebAssembly/binaryen) ) is recommended to be installed for release build optimization.

## Setup

```sh
npm create cloudflare ＜project dir＞ -- --template https://github.com/ohkami-rs/ohkami-templates/worker
```
```sh
cd ＜project dir＞
```

If you push the project to your GitHub repo, **you should add `wrangler.toml` into .gitignore**！

## Local dev

```sh
npm run dev
```

## Deploy

```sh
npx wrangler login
```

```sh
npm run deploy
```
If you register your workers.dev subdomain at this time, it takes some minutes for DNS records to update and it's good time to take a coffee break.
