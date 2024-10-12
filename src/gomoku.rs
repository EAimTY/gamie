//! Gomoku
//!
//! Check struct [`Gomoku`](https://docs.rs/gamie/*/gamie/gomoku/struct.Gomoku.html) for more information
//!
//! # Examples
//!
//! ```rust
//! # fn gomoku() {
//! use gamie::gomoku::{Gomoku, Player as GomokuPlayer};
//!
//! let mut game = Gomoku::new().unwrap();
//! game.place(GomokuPlayer::Player0, 7, 8).unwrap();
//! game.place(GomokuPlayer::Player1, 8, 7).unwrap();
//! // ...
//! # }
//! ```

extern crate alloc;

use alloc::boxed::Box;
use core::{convert::Infallible, iter};
use snafu::Snafu;

/// Gomoku
///
/// Passing an invalid position to a method will cause panic. Check the target position validity first when dealing with user input
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gomoku {
    board: [[Option<Player>; 15]; 15],
    next: Player,
    status: Status,
}

/// Players
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    Player0,
    Player1,
}

impl Player {
    /// Get the opposite player
    pub fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
        }
    }
}

/// Game status
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    Win(Player),
    Tie,
    InProgress,
}

impl Gomoku {
    /// Create a new Gomoku game.
    pub fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: [[None; 15]; 15],
            next: Player::Player0,
            status: Status::InProgress,
        })
    }

    /// Get a cell reference from the game board
    /// Panic when target position out of bounds
    pub fn get(&self, row: usize, col: usize) -> &Option<Player> {
        &self.board[row][col]
    }

    /// Place a piece on the board
    /// Panic when target position out of bounds
    pub fn place(&mut self, player: Player, row: usize, col: usize) -> Result<(), GomokuError> {
        if self.is_ended() {
            return Err(GomokuError::GameEnded);
        }

        if player != self.next {
            return Err(GomokuError::WrongPlayer);
        }

        if self.board[row][col].is_some() {
            return Err(GomokuError::OccupiedPosition);
        }

        self.board[row][col] = Some(player);
        self.next = self.next.other();

        self.check_game_status();

        Ok(())
    }

    /// Check if the game was end
    pub fn is_ended(&self) -> bool {
        self.status != Status::InProgress
    }

    /// Get the next player
    pub fn get_next_player(&self) -> Player {
        self.next
    }

    /// Get the winner of the game. Return `None` when the game is tied or not end yet
    pub fn get_winner(&self) -> Option<Player> {
        if let Status::Win(player) = self.status {
            Some(player)
        } else {
            None
        }
    }

    /// Get the game status
    pub fn get_game_status(&self) -> &Status {
        &self.status
    }

    fn check_game_status(&mut self) {
        for connectable in Self::get_connectable() {
            let mut last = None;
            let mut count = 0u8;

            for cell in connectable.map(|(row, col)| self.board[col][row]) {
                if cell != last {
                    last = cell;
                    count = 1;
                } else {
                    count += 1;
                    if count == 5 && cell.is_some() {
                        let winner = unsafe { cell.unwrap_unchecked() };
                        self.status = Status::Win(winner);
                        return;
                    }
                }
            }
        }

        if self.board.iter().flatten().all(|cell| cell.is_some()) {
            self.status = Status::Tie;
        }
    }

    fn get_connectable() -> impl Iterator<Item = Box<dyn Iterator<Item = (usize, usize)>>> {
        let horizontal = (0usize..15).map(move |row| {
            Box::new((0usize..15).map(move |col| (row, col)))
                as Box<dyn Iterator<Item = (usize, usize)>>
        });

        let vertical = (0usize..15).map(move |col| {
            Box::new((0usize..15).map(move |row| (row, col)))
                as Box<dyn Iterator<Item = (usize, usize)>>
        });

        let horizontal_upper_left_to_lower_right = (0usize..15).map(move |col| {
            Box::new(
                iter::successors(Some((0usize, col)), |(row, col)| Some((row + 1, col + 1)))
                    .take(15 - col),
            ) as Box<dyn Iterator<Item = (usize, usize)>>
        });

        let vertical_upper_left_to_lower_right = (0usize..15).map(move |row| {
            Box::new(
                iter::successors(Some((row, 0usize)), |(row, col)| Some((row + 1, col + 1)))
                    .take(15 - row),
            ) as Box<dyn Iterator<Item = (usize, usize)>>
        });

        let horizontal_upper_right_to_lower_left = (0usize..15).map(move |col| {
            Box::new(
                iter::successors(Some((0usize, col)), |(row, col)| {
                    col.checked_sub(1).map(|new_col| (row + 1, new_col))
                })
                .take(1 + col),
            ) as Box<dyn Iterator<Item = (usize, usize)>>
        });

        let vertical_upper_right_to_lower_left = (0usize..15).map(move |row| {
            Box::new(
                iter::successors(Some((row, 14usize)), |(row, col)| Some((row + 1, col - 1)))
                    .take(15 - row),
            ) as Box<dyn Iterator<Item = (usize, usize)>>
        });

        horizontal
            .chain(vertical)
            .chain(horizontal_upper_left_to_lower_right)
            .chain(vertical_upper_left_to_lower_right)
            .chain(horizontal_upper_right_to_lower_left)
            .chain(vertical_upper_right_to_lower_left)
    }
}

/// Errors that can occur when placing a piece on the board
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum GomokuError {
    #[snafu(display("Wrong player"))]
    WrongPlayer,
    #[snafu(display("Occupied position"))]
    OccupiedPosition,
    #[snafu(display("The game was already end"))]
    GameEnded,
}
