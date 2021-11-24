//! # boards
//! A Rust library provides several board game abstractions.
//!
//! This crate aims to provide simple yet adequate abstractions for several famous board games.
//! It is quite lightweight - came with dependencies fewer than the fingers on one hand, no AIs, just pure game implementions. It can be easily integrated into you projects.
//!
//! ## Usage
//! To use boards, you should enable modules you need in `Cargo.toml`. For example `tictactoe`:
//!
//! ```toml
//! [dependencies]
//! boards = { version = "0.1.0", features = ["tictactoe"] }
//! ```
//!
//! Now you can use the `tictactoe`:
//!
//! ```rust
//! use boards::tictactoe::{TicTacToe, TicTacToePlayer, TicTacToeStatus, TicTacToeError};
//!
//! let mut game = TicTacToe::new();
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
//! assert_eq!(game.status(), TicTacToeStatus::Tie);
//! ```
//!
//! Check the [docs](https://docs.rs/boards) for further usage information.
//!
//! ## Modules
//! Currently, the following modules are available:
//! - tictactoe: The Tic-Tac-Toe game.

#[cfg(feature = "tictactoe")]
pub mod tictactoe;
#[cfg(feature = "tictactoe")]
pub use tictactoe::*;
