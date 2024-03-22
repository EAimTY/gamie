#![doc = include_str!("../README.md")]
#![no_std]

#[cfg(feature = "connect_four")]
pub mod connect_four;

#[cfg(feature = "gomoku")]
pub mod gomoku;

#[cfg(feature = "minesweeper")]
pub mod minesweeper;

#[cfg(feature = "reversi")]
pub mod reversi;

#[cfg(feature = "tictactoe")]
pub mod tictactoe;
