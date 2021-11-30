//! The Connect Four game.
//!
//! Check out struct [`ConnectFour`](https://docs.rs/gamie/*/gamie/connect_four/struct.ConnectFour.html) for more information.
//!
//! # Examples
//!
//! ```rust
//! # fn connect_four() {
//! use gamie::connect_four::{ConnectFour, Player as ConnectFourPlayer};
//!
//! // ...
//! # }
//! ```

#[cfg(feature = "std")]
use std::{
    convert::Infallible,
    ops::{Index, IndexMut},
};

#[cfg(not(feature = "std"))]
use core::{
    convert::Infallible,
    ops::{Index, IndexMut},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use snafu::Snafu;

/// The Connect Four game.
///
/// If you pass an invalid position to a method, the game will panic. Remember to check the target position validity when dealing with user input.
#[derive(Clone, Debug)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub struct ConnectFour {
    pub board: [Column; 7],
    pub next: Player,
    pub state: GameState,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub struct Column {
    pub column: [Option<Player>; 6],
    pub occupied: usize,
}

impl Column {
    fn is_full(&self) -> bool {
        self.occupied == 6
    }

    fn push(&mut self, player: Player) {
        self.column[self.occupied] = Some(player);
        self.occupied += 1;
    }
}

impl Default for Column {
    fn default() -> Self {
        Self {
            column: [None; 6],
            occupied: 0,
        }
    }
}

impl Index<usize> for Column {
    type Output = Option<Player>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.column[index]
    }
}

impl IndexMut<usize> for Column {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.column[index]
    }
}

/// The game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Get the opposite player.
    pub fn other(self) -> Self {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

/// The game state.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub enum GameState {
    Win(Player),
    Tie,
    InProgress,
}

impl ConnectFour {
    /// Create a new Connect Four game.
    pub fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: Default::default(),
            next: Player::Player1,
            state: GameState::InProgress,
        })
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Option<Player> {
        &self.board[row][col]
    }

    /// Get a mutable cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut Option<Player> {
        &mut self.board[row][col]
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.state != GameState::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not end yet.
    pub fn winner(&self) -> Option<Player> {
        if let GameState::Win(player) = self.state {
            Some(player)
        } else {
            None
        }
    }

    /// Get the state of the game.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> Player {
        self.next
    }

    /// Put a piece in the game board.
    pub fn put(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        if self.is_ended() {
            return Err(ConnectFourError::GameEnded);
        }

        if player != self.next {
            return Err(ConnectFourError::WrongPlayer);
        }

        if self.board[col].is_full() {
            return Err(ConnectFourError::ColumnFull);
        }

        self.board[col].push(player);

        self.check_state();

        Ok(())
    }

    fn check_state(&mut self) {
        todo!();
    }
}

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum ConnectFourError {
    #[snafu(display("Wrong player"))]
    WrongPlayer,
    #[snafu(display("Full Column"))]
    ColumnFull,
    #[snafu(display("The game is already ended"))]
    GameEnded,
}

#[cfg(test)]
mod tests {
    // use crate::connect_four::*;

    #[test]
    fn test() {}
}
