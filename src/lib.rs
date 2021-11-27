//! # gamie
//! A Rust library provides abstractions for several classic tiny games.
//!
//! gamie aims to provide simple yet adequate abstractions for several classic tiny games.
//! It is quite lightweight - came with dependencies fewer than the fingers on one hand, no AI, just pure game implementions. It can be easily integrated into your projects.
//!
//! ## Usage
//! To use gamie, you should enable modules you need in `Cargo.toml`. For example `tictactoe`:
//!
//! ```toml
//! [dependencies]
//! gamie = { version = "0.3.0", features = ["tictactoe"] }
//! ```
//!
//! Now you can use the `tictactoe`:
//!
//! ```rust
//! use gamie::tictactoe::*;
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
//! assert_eq!(game.state(), &TicTacToeState::Tie);
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

#[cfg(feature = "minesweeper")]
pub mod minesweeper;

#[cfg(feature = "reversi")]
pub mod reversi;

#[cfg(feature = "tictactoe")]
pub mod tictactoe;
