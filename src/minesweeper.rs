//! The Minesweeper game.
//!
//! # Examples
//!
//! ```rust
//! use gamie::minesweeper::*;
//!
//! # fn minesweeper() {
//! # }
//! ```

/// The Minesweeper game itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Minesweeper {}

/// The Minesweeper game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinesweeperStatus {
    Win,
    Tie,
    InProgress,
}

use thiserror::Error;

/// Errors that can occur when clicking a cell on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum MinesweeperError {
    #[error("Position out of bounds")]
    OutOfBounds,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::minesweeper::*;

    #[test]
    fn test() {}
}
