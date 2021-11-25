//! The Reversi (Othello) game.
//!
//! # Examples
//!
//! ```rust
//! use gamie::reversi::*;
//!
//! # fn reversi() {
//! # }
//! ```

/// The Reversi game itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reversi {
    pub board: [[Option<ReversiPlayer>; 8]; 8],
    pub next: ReversiPlayer,
    pub status: ReversiStatus,
}

/// The Reversi game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReversiPlayer {
    Black,
    White,
}

impl ReversiPlayer {
    /// Get the opposite player.
    pub fn other(self) -> Self {
        match self {
            ReversiPlayer::Black => ReversiPlayer::White,
            ReversiPlayer::White => ReversiPlayer::Black,
        }
    }
}

/// The Reversi game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReversiStatus {
    Win(ReversiPlayer),
    Tie,
    InProgress,
}

impl Reversi {
    /// Create a new Reversi game.
    pub fn new() -> Self {
        Self {
            board: [[None; 8]; 8],
            next: ReversiPlayer::Black,
            status: ReversiStatus::InProgress,
        }
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.status != ReversiStatus::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not ended.
    pub fn winner(&self) -> Option<ReversiPlayer> {
        if let ReversiStatus::Win(player) = self.status {
            Some(player)
        } else {
            None
        }
    }

    /// Get the status of the game.
    pub fn status(&self) -> ReversiStatus {
        self.status
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> ReversiPlayer {
        self.next
    }

    /// Get the board.
    pub fn board(&self) -> &[[Option<ReversiPlayer>; 8]; 8] {
        &self.board
    }

    /// Place a piece on the board.
    pub fn place(
        &mut self,
        player: ReversiPlayer,
        row: usize,
        col: usize,
    ) -> Result<ReversiStatus, ReversiError> {
        if self.is_ended() {
            return Err(ReversiError::GameEnded);
        }

        if player != self.next {
            return Err(ReversiError::WrongPlayer);
        }

        if row > 7 || col > 7 {
            return Err(ReversiError::OutOfBounds);
        }

        todo!();
    }

    fn check_status(&self) -> ReversiStatus {
        todo!();
    }
}

use thiserror::Error;

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ReversiError {
    #[error("Wrong player")]
    WrongPlayer,
    #[error("Position out of bounds")]
    OutOfBounds,
    #[error("Invalid position")]
    InvalidPosition,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::reversi::*;

    #[test]
    fn test() {}
}
