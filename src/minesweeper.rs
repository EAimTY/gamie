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
pub struct Minesweeper {
    pub board: Vec<MinesweeperCell>,
    pub height: usize,
    pub width: usize,
    pub status: MinesweeperStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MinesweeperCell {
    pub is_mine: bool,
    pub mines_around: usize,
    pub is_revealed: bool,
    pub is_flagged: bool,
}

impl MinesweeperCell {
    pub fn new(is_mine: bool) -> Self {
        Self {
            is_mine,
            mines_around: 0,
            is_revealed: false,
            is_flagged: false,
        }
    }
}

impl Minesweeper {
    pub fn new(height: usize, width: usize, mines: usize) -> Self {
        let board = itertools::repeat_n(MinesweeperCell::new(true), mines)
            .chain(itertools::repeat_n(
                MinesweeperCell::new(false),
                height * width - mines,
            ))
            .collect();

        let mut minesweeper = Self {
            board,
            height,
            width,
            status: MinesweeperStatus::InProgress,
        };
        minesweeper.randomize();

        minesweeper
    }

    pub fn randomize(&mut self) {
        use rand::distributions::{Distribution, Uniform};

        let mut rng = rand::thread_rng();
        let range = Uniform::from(0..self.height * self.width);

        for idx in 0..self.height * self.width {
            self.board.swap(idx, range.sample(&mut rng));
        }

        self.count_for_all_cells();
    }

    pub fn get(&self, row: usize, col: usize) -> Result<MinesweeperCell, MinesweeperError> {
        if row >= self.height || col >= self.width {
            return Err(MinesweeperError::OutOfBounds);
        }

        Ok(self.board[row * self.width + col])
    }

    pub fn click_unrevealed(&mut self, row: usize, col: usize) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        todo!();
    }

    pub fn click_revealed(&mut self, row: usize, col: usize, auto_flag: bool) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        todo!();
    }

    pub fn flag(&mut self, row: usize, col: usize) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        todo!();
    }

    fn check_game_status(&self) -> MinesweeperStatus {
        todo!();
    }

    fn check_position_validity(&self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        if self.status != MinesweeperStatus::InProgress {
            return Err(MinesweeperError::GameEnded);
        }

        if row >= self.height || col >= self.width {
            return Err(MinesweeperError::OutOfBounds);
        }

        Ok(())
    }

    fn count_for_all_cells(&mut self) {
        for idx in 0..self.height * self.width {
            let count = self
                .positions_around_from(idx)
                .filter(|arnd_idx| self.board[*arnd_idx].is_mine)
                .count();

            self.board[idx].mines_around = count;
        }
    }

    fn positions_around_from(&self, idx: usize) -> impl Iterator<Item = usize> + '_ {
        let (row, col) = ((idx / self.width) as i128, (idx % self.width) as i128);

        let around = [
            (row - 1, col - 1),
            (row - 1, col),
            (row - 1, col + 1),
            (row, col - 1),
            (row, col + 1),
            (row + 1, col - 1),
            (row + 1, col),
            (row + 1, col + 1),
        ];

        around
            .into_iter()
            .filter(|(row, col)| {
                *row >= 0 && *col >= 0 && *row < self.height as i128 && *col < self.width as i128
            })
            .map(|(row, col)| row as usize * self.width + col as usize)
    }
}

/// The Minesweeper game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinesweeperStatus {
    Win,
    Exploded(usize, usize),
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
