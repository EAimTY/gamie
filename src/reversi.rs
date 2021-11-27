//! The Reversi game.
//!
//! Check out struct [`Reversi`](https://docs.rs/gamie/*/gamie/reversi/struct.Reversi.html) for more information.
//!
//! # Examples
//!
//! ```rust
//! # fn reversi() {
//! use gamie::reversi::*;
//!
//! let mut game = Reversi::new().unwrap();
//!
//! game.place(2, 4, ReversiPlayer::Black).unwrap();
//!
//! // The next player may not be able to place the piece in any position, so remember to check `get_next_player()`.
//! assert_eq!(game.get_next_player(), ReversiPlayer::White);
//!
//! game.place(2, 3, ReversiPlayer::White).unwrap();
//!
//! // ...
//! # }
//! ```

#[cfg(not(feature = "std"))]
use core::{cmp::Ordering, convert::Infallible};

#[cfg(feature = "std")]
use std::{cmp::Ordering, convert::Infallible};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The Reversi game.
/// If you pass an invalid position to a method, the game will panic. Remember to check the target position validity when dealing with user input.
#[derive(Clone, Debug)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub struct Reversi {
    pub board: [[Option<ReversiPlayer>; 8]; 8],
    pub next: ReversiPlayer,
    pub state: ReversiState,
}

/// The game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
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

/// The game state.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(feature = "serde")]
#[derive(Deserialize, Serialize)]
pub enum ReversiState {
    Win(ReversiPlayer),
    Tie,
    InProgress,
}

impl Reversi {
    /// Create a new Reversi game.
    pub fn new() -> Result<Self, Infallible> {
        let mut board = [[None; 8]; 8];
        board[3][3] = Some(ReversiPlayer::Black);
        board[4][4] = Some(ReversiPlayer::Black);
        board[3][4] = Some(ReversiPlayer::White);
        board[4][3] = Some(ReversiPlayer::White);

        Ok(Self {
            board,
            next: ReversiPlayer::Black,
            state: ReversiState::InProgress,
        })
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Option<ReversiPlayer> {
        &self.board[row][col]
    }

    /// Get a mutable cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut Option<ReversiPlayer> {
        &mut self.board[row][col]
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.state != ReversiState::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not ended yet.
    pub fn winner(&self) -> Option<ReversiPlayer> {
        if let ReversiState::Win(player) = self.state {
            Some(player)
        } else {
            None
        }
    }

    /// Get the state of the game.
    pub fn state(&self) -> &ReversiState {
        &self.state
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> ReversiPlayer {
        self.next
    }

    /// Place a piece on the board.
    /// Panic if the target position is out of bounds.
    pub fn place(
        &mut self,
        row: usize,
        col: usize,
        player: ReversiPlayer,
    ) -> Result<(), ReversiError> {
        self.simple_check_position_validity(row, col, player)?;

        let mut flipped = false;

        for dir in Direction::iter() {
            if let Some((to_row, to_col)) =
                self.check_occupied_line_in_direction(row, col, dir, player)
            {
                self.flip(row, col, to_row, to_col, dir, player);
                flipped = true;
            }
        }

        if flipped {
            self.next = player.other();

            if !self.can_player_move(player.other()) {
                self.next = player;

                if !self.can_player_move(player) {
                    self.check_state();
                }
            }

            Ok(())
        } else {
            Err(ReversiError::InvalidPosition)
        }
    }

    /// Check if a position is valid. Return the reason as `Err(ReversiError)` if it is not.
    /// Panic if the target position is out of bounds.
    pub fn check_position_validity(
        &self,
        row: usize,
        col: usize,
        player: ReversiPlayer,
    ) -> Result<(), ReversiError> {
        self.simple_check_position_validity(row, col, player)?;

        if Direction::iter()
            .map(|dir| self.check_occupied_line_in_direction(row, col, dir, player))
            .any(|o| o.is_some())
        {
            Ok(())
        } else {
            Err(ReversiError::InvalidPosition)
        }
    }

    fn simple_check_position_validity(
        &self,
        row: usize,
        col: usize,
        player: ReversiPlayer,
    ) -> Result<(), ReversiError> {
        if self.is_ended() {
            return Err(ReversiError::GameEnded);
        }

        if player != self.next {
            return Err(ReversiError::WrongPlayer);
        }

        if self.board[row][col].is_some() {
            return Err(ReversiError::PositionOccupied);
        }

        Ok(())
    }

    fn can_player_move(&self, player: ReversiPlayer) -> bool {
        for row in 0..8 {
            for col in 0..8 {
                if self.board[row][col].is_none()
                    && self.check_position_validity(row, col, player).is_ok()
                {
                    return true;
                }
            }
        }

        false
    }

    fn check_state(&mut self) {
        let mut black_count = 0;
        let mut white_count = 0;

        for cell in self.board.iter().flatten().flatten() {
            match cell {
                ReversiPlayer::Black => black_count += 1,
                ReversiPlayer::White => white_count += 1,
            }
        }

        self.state = match black_count.cmp(&white_count) {
            Ordering::Less => ReversiState::Win(ReversiPlayer::White),
            Ordering::Equal => ReversiState::Tie,
            Ordering::Greater => ReversiState::Win(ReversiPlayer::Black),
        };
    }

    fn flip(
        &mut self,
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        dir: Direction,
        player: ReversiPlayer,
    ) {
        self.iter_positions_in_direction_from(from_row, from_col, dir)
            .take_while(|(row, col)| *row != to_row || *col != to_col)
            .for_each(|(row, col)| {
                self.board[row][col] = Some(player);
            });
    }

    fn check_occupied_line_in_direction(
        &self,
        row: usize,
        col: usize,
        dir: Direction,
        player: ReversiPlayer,
    ) -> Option<(usize, usize)> {
        let mut pos = self.iter_positions_in_direction_from(row, col, dir);

        pos.next();

        let first = if let Some(pos) = pos.next() {
            pos
        } else {
            return None;
        };

        if self.board[first.0][first.1] != Some(player.other()) {
            return None;
        }

        for (row, col) in pos {
            match self.board[row][col] {
                Some(piece) if piece == player.other() => continue,
                Some(_) => return Some((row, col)),
                None => return None,
            }
        }

        None
    }

    fn iter_positions_in_direction_from(
        &self,
        row: usize,
        col: usize,
        dir: Direction,
    ) -> impl Iterator<Item = (usize, usize)> {
        itertools::iterate((row, col), move |(row, col)| {
            let (offset_row, offset_col) = dir.as_offset();
            (
                (*row as i8 + offset_row) as usize,
                (*col as i8 + offset_col) as usize,
            )
        })
        .take_while(|(row, col)| *row < 8 && *col < 8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Upper,
    UpperRight,
    Right,
    LowerRight,
    Lower,
    LowerLeft,
    Left,
    UpperLeft,
}

impl Direction {
    fn as_offset(&self) -> (i8, i8) {
        match self {
            Direction::Upper => (-1, 0),
            Direction::UpperRight => (-1, 1),
            Direction::Right => (0, 1),
            Direction::LowerRight => (1, 1),
            Direction::Lower => (1, 0),
            Direction::LowerLeft => (1, -1),
            Direction::Left => (0, -1),
            Direction::UpperLeft => (-1, -1),
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        [
            Direction::Upper,
            Direction::UpperRight,
            Direction::Right,
            Direction::LowerRight,
            Direction::Lower,
            Direction::LowerLeft,
            Direction::Left,
            Direction::UpperLeft,
        ]
        .into_iter()
    }
}

use thiserror::Error;

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ReversiError {
    #[error("Wrong player")]
    WrongPlayer,
    #[error("Position already occupied")]
    PositionOccupied,
    #[error("Invalid position")]
    InvalidPosition,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::reversi::*;

    #[test]
    fn test() {
        let mut game = Reversi::new().unwrap();

        assert_eq!(game.place(2, 4, ReversiPlayer::Black), Ok(()));

        assert_eq!(game.place(2, 3, ReversiPlayer::White), Ok(()));

        assert_eq!(
            game.place(2, 6, ReversiPlayer::White),
            Err(ReversiError::WrongPlayer)
        );

        assert_eq!(
            game.place(2, 6, ReversiPlayer::Black),
            Err(ReversiError::InvalidPosition)
        );
    }
}
