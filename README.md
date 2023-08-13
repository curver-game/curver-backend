# Curver Backend

<img width="1728" alt="Screen Shot 2023-08-14 at 02 06 45" src="https://github.com/curver-game/curver-backend/assets/23079646/17c3ae1c-7a21-4d3e-97c1-1809128b5500">

Curver is a tron-clone game. The game is played on a 2D plane. Every player moves continously and leave trails behind. If any player hits the trails or go out of the map's bounds, they get eliminated. The goal of the game is to be the last player standing.

This repository contains a multi-threaded websocket server written in Rust for the game.

## How to build?

1. [Install cargo](https://doc.rust-lang.org/book/ch01-01-installation.html) if you don't have it.
2. Run `cargo build --release --locked`.
3. An executable binary will be created and placed under the `target/` folder.

## How to run?

1. [Install cargo](https://doc.rust-lang.org/book/ch01-01-installation.html) if you don't have it.
2. Run `cargo run` to run the server as a debug variant. The server listens to `0.0.0.:8080` by default.
