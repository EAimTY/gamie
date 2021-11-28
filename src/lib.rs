//! # gamie
//! A Rust library provides abstractions for several classic tiny games.
//!
//! gamie aims to provide simple yet adequate abstractions for several classic tiny games.
//!
//! gamie is quite lightweight - came with few dependencies, no AI, just pure game implementions. It can be easily integrated into your projects.
//!
//! ## Usage
//! To use gamie, you should enable modules you need in `Cargo.toml`. For example `tictactoe`:
//!
//! ```toml
//! [dependencies]
//! gamie = { version = "*", features = ["tictactoe"] }
//! ```
//!
//! Now you can use the `tictactoe`:
//!
//! ```rust
//! use gamie::tictactoe::{TicTacToe, Player as TicTacToePlayer, GameState as TicTacToeGameState};
//!
//! # fn tictactoe() {
//! let mut game = TicTacToe::new().unwrap();
//! game.place(TicTacToePlayer::X, 1, 1).unwrap();
//! game.place(TicTacToePlayer::O, 0, 0).unwrap();
//! game.place(TicTacToePlayer::X, 0, 2).unwrap();
//! game.place(TicTacToePlayer::O, 2, 0).unwrap();
//! game.place(TicTacToePlayer::X, 1, 0).unwrap();
//! game.place(TicTacToePlayer::O, 1, 2).unwrap();
//! game.place(TicTacToePlayer::X, 2, 1).unwrap();
//! game.place(TicTacToePlayer::O, 0, 1).unwrap();
//! game.place(TicTacToePlayer::X, 2, 2).unwrap();
//! assert!(game.is_ended());
//! assert_eq!(game.state(), &TicTacToeGameState::Tie);
//! # }
//! ```
//!
//! Check the [docs](https://docs.rs/gamie) for further usage information.
//!
//! ## Modules
//! Currently, the following modules are available:
//!
//! - [minesweeper](https://docs.rs/gamie/*/gamie/minesweeper)
//! - [reversi](https://docs.rs/gamie/*/gamie/reversi)
//! - [tictactoe](https://docs.rs/gamie/*/gamie/tictactoe)
//!
//! ## Serialize / Deserialize
//! Bring in the `serde` feature to enable serialization and deserialization.
//!
//! ```toml
//! [dependencies]
//! gamie = { version = "*", features = ["serde", "tictactoe"] }
//! ```
//!
//! ## no_std
//! This crate can run flawlessly on bare metal.
//! Opt out the `std` feature by disabling `default-features` in `Cargo.toml` to remove the Rust standard library dependency.
//!
//! ```toml
//! [dependencies]
//! gamie = { version = "*", features = ["tictactoe"], default-features = false }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(feature = "minesweeper")]
pub mod minesweeper;

#[cfg(feature = "reversi")]
pub mod reversi;

#[cfg(feature = "tictactoe")]
pub mod tictactoe;
