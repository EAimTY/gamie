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
//! let mut game = ConnectFour::new().unwrap();
//! game.put(3, ConnectFourPlayer::Player0).unwrap();
//! game.put(2, ConnectFourPlayer::Player1).unwrap();
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

/// The column of the game board.
///
/// This is a stack-vector-like struct. You can access its inner elements by using index directly.
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
    Player0,
    Player1,
}

impl Player {
    /// Get the opposite player.
    pub fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
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
            next: Player::Player0,
            state: GameState::InProgress,
        })
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Option<Player> {
        &self.board[5 - row][col]
    }

    /// Get a mutable cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut Option<Player> {
        &mut self.board[5 - row][col]
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

    /// Put a piece into the game board.
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
        self.next = self.next.other();

        self.check_state();

        Ok(())
    }

    fn check_state(&mut self) {
        for connectable in Self::get_connectable() {
            let mut last = None;
            let mut count = 0u8;

            for cell in connectable.iter().map(|(row, col)| self.board[*col][*row]) {
                if cell != last {
                    last = cell;
                    count = 1;
                } else {
                    count += 1;
                    if count == 4 && cell.is_some() {
                        self.state = GameState::Win(cell.unwrap());
                        return;
                    }
                }
            }
        }

        if (0..7).all(|col| !self.board[col][5].is_some()) {
            self.state = GameState::Tie;
        }
    }

    fn get_connectable() -> impl Iterator<Item = &'static [(usize, usize)]> {
        let connectable: &[&[(usize, usize)]] = &[
            &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6)],
            &[(1, 0), (1, 1), (1, 2), (1, 3), (1, 4), (1, 5), (1, 6)],
            &[(2, 0), (2, 1), (2, 2), (2, 3), (2, 4), (2, 5), (2, 6)],
            &[(3, 0), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5), (3, 6)],
            &[(4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (4, 5), (4, 6)],
            &[(5, 0), (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6)],
            &[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0)],
            &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            &[(0, 2), (1, 2), (2, 2), (3, 2), (4, 2), (5, 2)],
            &[(0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3)],
            &[(0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (5, 4)],
            &[(0, 5), (1, 5), (2, 5), (3, 5), (4, 5), (5, 5)],
            &[(0, 6), (1, 6), (2, 6), (3, 6), (4, 6), (5, 6)],
            &[(0, 3), (1, 2), (2, 1), (3, 0)],
            &[(0, 4), (1, 3), (2, 2), (3, 1), (4, 0)],
            &[(0, 5), (1, 4), (2, 3), (3, 2), (4, 1), (5, 0)],
            &[(0, 6), (1, 5), (2, 4), (3, 3), (4, 2), (5, 1)],
            &[(1, 6), (2, 5), (3, 4), (4, 3), (5, 2)],
            &[(2, 6), (3, 5), (4, 4), (5, 3)],
            &[(2, 0), (3, 1), (4, 2), (5, 3)],
            &[(1, 0), (2, 1), (3, 2), (4, 3), (5, 4)],
            &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (5, 5)],
            &[(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 6)],
            &[(0, 2), (1, 3), (2, 4), (3, 5), (4, 6)],
            &[(0, 3), (1, 4), (2, 5), (3, 6)],
        ];

        connectable.iter().map(|pos| *pos)
    }
}

/// Errors that can occur when putting a piece into the board.
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum ConnectFourError {
    #[snafu(display("Wrong player"))]
    WrongPlayer,
    #[snafu(display("Full Column"))]
    ColumnFull,
    #[snafu(display("The game is already end"))]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::connect_four::*;

    #[test]
    fn test() {
        let mut game = ConnectFour::new().unwrap();
        game.put(3, Player::Player0).unwrap();
        game.put(2, Player::Player1).unwrap();
        game.put(2, Player::Player0).unwrap();
        game.put(1, Player::Player1).unwrap();
        game.put(1, Player::Player0).unwrap();
        game.put(0, Player::Player1).unwrap();
        game.put(3, Player::Player0).unwrap();
        game.put(0, Player::Player1).unwrap();
        game.put(1, Player::Player0).unwrap();
        game.put(6, Player::Player1).unwrap();
        game.put(2, Player::Player0).unwrap();
        game.put(6, Player::Player1).unwrap();
        game.put(3, Player::Player0).unwrap();
        game.put(5, Player::Player1).unwrap();
        game.put(0, Player::Player0).unwrap();
        assert_eq!(Some(Player::Player0), game.winner());
    }
}
