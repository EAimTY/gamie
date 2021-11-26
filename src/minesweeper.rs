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

/// The Minesweeper game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MinesweeperStatus {
    Win,
    Exploded(usize, usize),
    InProgress,
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

        self.update_around_mine_count();
    }

    pub fn click_unrevealed(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        if self.board[row * self.width + col].is_flagged {
            return Err(MinesweeperError::AlreadyFlagged);
        }

        if self.board[row * self.width + col].is_mine {
            self.status = MinesweeperStatus::Exploded(row, col);
            return Ok(self.status);
        }

        self.reveal_from(row, col);
        self.status = self.check_game_status();

        Ok(self.status)
    }

    pub fn click_revealed(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        todo!();
    }

    pub fn toggle_flag(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        if self.board[row * self.width + col].is_revealed {
            return Err(MinesweeperError::AlreadyRevealed);
        }

        self.board[row * self.width + col].is_flagged =
            !self.board[row * self.width + col].is_flagged;

        self.status = self.check_game_status();

        Ok(self.status)
    }

    pub fn get_cell_neighbors(
        &self,
        row: usize,
        col: usize,
    ) -> Result<MinesweeperCellsAround, MinesweeperError> {
        self.check_position_validity(row, col)?;
        Ok(self.get_cell_neighbors_by_coords(row, col))
    }

    fn reveal_from(&mut self, row: usize, col: usize) {
        if self.board[row * self.width + col].mines_around != 0 {
            self.board[row * self.width + col].is_revealed = true;
        } else {
            use std::collections::VecDeque;

            let mut cells_to_reveal = VecDeque::new();
            cells_to_reveal.push_back(row * self.width + col);

            while let Some(cell_idx) = cells_to_reveal.pop_front() {
                for neighbor_idx in self.get_cell_neighbors_by_idx(cell_idx).iter_by_idx() {
                    self.board[neighbor_idx].is_revealed = true;

                    if self.board[neighbor_idx].mines_around == 0 {
                        cells_to_reveal.push_back(neighbor_idx);
                    }
                }
            }
        }
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

    fn update_around_mine_count(&mut self) {
        for idx in 0..self.height * self.width {
            let count = self
                .get_cell_neighbors_by_idx(idx)
                .iter_by_idx()
                .filter(|arnd_idx| self.board[*arnd_idx].is_mine)
                .count();

            self.board[idx].mines_around = count;
        }
    }

    fn get_cell_neighbors_by_coords(&self, row: usize, col: usize) -> MinesweeperCellsAround {
        MinesweeperCellsAround::new(row, col, self.height, self.width)
    }

    fn get_cell_neighbors_by_idx(&self, idx: usize) -> MinesweeperCellsAround {
        self.get_cell_neighbors_by_coords(idx / self.width, idx % self.width)
    }
}

#[derive(Clone, Debug)]
pub struct MinesweeperCellsAround {
    around: [(i128, i128); 8],
    board_height: i128,
    board_width: i128,
    offset: usize,
}

impl Iterator for MinesweeperCellsAround {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        self.around[self.offset..]
            .iter()
            .enumerate()
            .filter(|(_, (row, col))| {
                *row >= 0 && *col >= 0 && *row < self.board_height && *col < self.board_width
            })
            .next()
            .map(|(idx, (row, col))| {
                self.offset += idx + 1;
                (*row as usize, *col as usize)
            })
    }
}

impl MinesweeperCellsAround {
    fn new(row: usize, col: usize, board_height: usize, board_width: usize) -> Self {
        let (row, col, board_height, board_width) = (
            row as i128,
            col as i128,
            board_height as i128,
            board_width as i128,
        );

        MinesweeperCellsAround {
            around: [
                (row - 1, col - 1),
                (row - 1, col),
                (row - 1, col + 1),
                (row, col - 1),
                (row, col + 1),
                (row + 1, col - 1),
                (row + 1, col),
                (row + 1, col + 1),
            ],
            board_height,
            board_width,
            offset: 0,
        }
    }

    fn iter_by_idx(self) -> impl Iterator<Item = usize> {
        let board_width = self.board_width;
        self.map(move |(row, col)| row * board_width as usize + col)
    }
}

use thiserror::Error;

/// Errors that can occur when clicking an unrevealed cell on the board.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MinesweeperError {
    #[error("Position out of bounds")]
    OutOfBounds,
    #[error("Clicked an already flagged cell")]
    AlreadyFlagged,
    #[error("Clicked an already revealed cell")]
    AlreadyRevealed,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    // use crate::minesweeper::*;

    #[test]
    fn test() {}
}
