//! The Reversi game.
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
        let mut board = [[None; 8]; 8];
        board[3][3] = Some(ReversiPlayer::Black);
        board[4][4] = Some(ReversiPlayer::Black);
        board[3][4] = Some(ReversiPlayer::White);
        board[4][3] = Some(ReversiPlayer::White);

        Self {
            board,
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
        row: usize,
        col: usize,
        player: ReversiPlayer,
    ) -> Result<ReversiStatus, ReversiError> {
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
                    self.status = self.check_status();
                }
            }

            Ok(self.status)
        } else {
            Err(ReversiError::InvalidPosition)
        }
    }

    /// Check if a position is valid. Return the reason as `Err(ReversiError)` if it is not.
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

        if row > 7 || col > 7 {
            return Err(ReversiError::OutOfBounds);
        }

        if self.board[row][col].is_some() {
            return Err(ReversiError::PositionOccupied);
        }

        Ok(())
    }

    fn can_player_move(&self, player: ReversiPlayer) -> bool {
        self.board
            .iter()
            .flatten()
            .enumerate()
            .map(|(idx, cell)| (idx / 8, idx % 8, cell))
            .filter(|(_, _, cell)| cell.is_none())
            .any(|(row, col, _)| self.check_position_validity(row, col, player).is_ok())
    }

    fn check_status(&self) -> ReversiStatus {
        let mut black_count = 0;
        let mut white_count = 0;

        for cell in self.board.iter().flatten().flatten() {
            match cell {
                ReversiPlayer::Black => black_count += 1,
                ReversiPlayer::White => white_count += 1,
            }
        }

        use std::cmp::Ordering;

        match black_count.cmp(&white_count) {
            Ordering::Less => ReversiStatus::Win(ReversiPlayer::White),
            Ordering::Equal => ReversiStatus::Tie,
            Ordering::Greater => ReversiStatus::Win(ReversiPlayer::Black),
        }
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

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
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
}

use thiserror::Error;

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum ReversiError {
    #[error("Wrong player")]
    WrongPlayer,
    #[error("Position out of bounds")]
    OutOfBounds,
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
    fn test() {}
}
