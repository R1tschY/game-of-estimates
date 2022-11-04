# Game Of Estimates
*Simple web [planning poker](https://en.wikipedia.org/wiki/Planning_poker) game*

Source code for https://game-of-estimates.richardliebscher.de/

## Features

* Viewers (Players that do not vote)
* Players can give themselves a name (stored in browser storage)

---

* No database required
* No personal data is stored


## Build from source
* Install NPM and Rust toolchain
* Build Frontend: `cd frontend && npm install && npm run build`
* Build Backend: `cargo build --release`
* Deploy `target/release/game-of-estimates` to your server

## Usage

Run `game-of-estimates` with environment variables:
* `GOE_LISTEN_ADDR`: address the service should listen to (for example: `0.0.0.0:5500`)
