# Game Of Estimates
*Simple web [planning poker](https://en.wikipedia.org/wiki/Planning_poker) game*

Source code for https://game-of-estimates.richardliebscher.de/

## Features

* Viewers (Players that do not vote)
* Players can give themselves a name (stored in browser storage)

* No database required
* No personal data is stored

## Usage

### Backend
* Install Rust toolchain
* `cargo build --release`
* Deploy `target/release/game-of-estimates`
* Run `game-of-estimates` with enviroment variables:
  * `GOE_WEBSOCKET_ADDR`: address the service should listen to (for example: `0.0.0.0:5500`)
  
### Frontend:
* Install NPM
* Set enviroment variable `GOE_WEBSOCKET_URL` to Websocket url, where backend service is accessible
  * Through HTTPS proxy: `wss://server/path`
  * Plain HTTP: `ws://server/path`
* `cd frontend && npm run build`
* Deploy `frontend/public` folder to your server
* Configure your server to always serve `index.html`
  * `.htaccess` for Apache HTTPD already exists
