# gamie

A Rust library providing abstractions for several classic tiny games.

[![Version](https://img.shields.io/crates/v/gamie.svg?style=flat)](https://crates.io/crates/gamie)
[![Documentation](https://img.shields.io/badge/docs-release-brightgreen.svg?style=flat)](https://docs.rs/gamie)
[![License](https://img.shields.io/crates/l/gamie.svg?style=flat)](https://github.com/EAimTY/gamie/blob/master/LICENSE)

gamie provides simple, well-designed abstractions for several classic tiny games.

gamie has minimal dependencies and can be easily integrated into your projects.

## Usage

To use gamie, enable the desired feature flags in `Cargo.toml`. For example, to use `tictactoe`:

```toml
[dependencies]
gamie = { version = "0.14.0", features = ["tictactoe"] }
```

Now you can use the Tic-Tac-Toe game abstraction:

```rust,ignore
use gamie::tictactoe::{Game, Status};

let mut game = Game::new().unwrap();
game.put(1, 1).unwrap();  // Player0 at center
game.put(0, 0).unwrap();  // Player1 at top-left
game.put(0, 2).unwrap();  // Player0 at top-right
game.put(2, 0).unwrap();  // Player1 at bottom-left
game.put(1, 0).unwrap();  // Player0 at middle-left
game.put(1, 2).unwrap();  // Player1 at middle-right
game.put(2, 1).unwrap();  // Player0 at bottom-center
game.put(0, 1).unwrap();  // Player1 at top-center
game.put(2, 2).unwrap();  // Player0 at bottom-right
assert_eq!(game.status(), &Status::Draw);
```

Check the [docs](https://docs.rs/gamie) for further information.

## Modules

Currently, the following modules are available:

- [connect_four](https://docs.rs/gamie/latest/gamie/connect_four)
- [gomoku](https://docs.rs/gamie/latest/gamie/gomoku)
- [minesweeper](https://docs.rs/gamie/latest/gamie/minesweeper)
- [reversi](https://docs.rs/gamie/latest/gamie/reversi)
- [tictactoe](https://docs.rs/gamie/latest/gamie/tictactoe)

## Serialization

Enable the `serde` feature to add serialization and deserialization support for game state types.

Enable the `rkyv` feature to add `rkyv` archive, serialize, and deserialize support. Generated
archived and resolver types are re-exported from each game module's `rkyv` submodule.

## no_std

This crate is `no_std` by default and runs flawlessly on bare metal.

## License

Licensed under either of:

- Apache License, Version 2.0, in [LICENSE-APACHE](LICENSE-APACHE)
- MIT license, in [LICENSE-MIT](LICENSE-MIT)

at your option.
