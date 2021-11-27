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
    pub adjacent_mine: usize,
    pub is_revealed: bool,
    pub is_flagged: bool,
}

impl MinesweeperCell {
    pub fn new(is_mine: bool) -> Self {
        Self {
            is_mine,
            adjacent_mine: 0,
            is_revealed: false,
            is_flagged: false,
        }
    }
}

/// The Minesweeper game status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinesweeperStatus {
    Win,
    Exploded(Vec<(usize, usize)>),
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

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.status != MinesweeperStatus::InProgress
    }

    /// Get the status of the game.
    pub fn status(&self) -> &MinesweeperStatus {
        &self.status
    }

    pub fn get(&self, row: usize, col: usize) -> Result<&MinesweeperCell, MinesweeperError> {
        self.check_position_validity(row, col)?;
        Ok(&self.board[row * self.width + col])
    }

    pub fn click(
        &mut self,
        row: usize,
        col: usize,
        allow_click_revealed: bool,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        if !self.board[row * self.width + col].is_revealed {
            Ok(self.click_unrevealed(row, col)?)
        } else if allow_click_revealed {
            Ok(self.click_revealed(row, col)?)
        } else {
            return Err(MinesweeperError::AlreadyRevealed);
        }
    }

    pub fn toggle_flag(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        self.check_position_validity(row, col)?;

        if self.board[row * self.width + col].is_revealed {
            return Err(MinesweeperError::AlreadyRevealed);
        }

        self.board[row * self.width + col].is_flagged =
            !self.board[row * self.width + col].is_flagged;

        self.status = self.check_game_status();

        Ok(&self.status)
    }

    fn click_unrevealed(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        if self.board[row * self.width + col].is_flagged {
            return Err(MinesweeperError::AlreadyFlagged);
        }

        if self.board[row * self.width + col].is_mine {
            self.status = MinesweeperStatus::Exploded(vec![(row, col)]);
            return Ok(&self.status);
        }

        self.reveal_from(row * self.width + col);
        self.status = self.check_game_status();

        Ok(&self.status)
    }

    fn click_revealed(
        &mut self,
        row: usize,
        col: usize,
    ) -> Result<&MinesweeperStatus, MinesweeperError> {
        if self.board[row * self.width + col].adjacent_mine > 0 {
            let mut adjacent_all = 0;
            let mut adjacent_revealed = 0;
            let mut adjacent_flagged = 0;

            self.get_adjacent_cells_by_coords(row, col)
                .iter_by_idx()
                .map(|idx| self.board[idx])
                .for_each(|cell| {
                    adjacent_all += 1;

                    if cell.is_revealed {
                        adjacent_revealed += 1;
                    } else if cell.is_flagged {
                        adjacent_flagged += 1;
                    }
                });

            let adjacent_unrevealed = adjacent_all - adjacent_revealed - adjacent_flagged;

            if adjacent_unrevealed > 0
                && adjacent_flagged == self.board[row * self.width + col].adjacent_mine
            {
                let mut exploded = None;

                self.get_adjacent_cells_by_coords(row, col)
                    .iter_by_idx()
                    .for_each(|idx| {
                        if !self.board[idx].is_flagged && !self.board[idx].is_revealed {
                            if self.board[idx].is_mine {
                                self.board[idx].is_revealed = true;

                                match exploded {
                                    None => exploded = Some(vec![(row, col)]),
                                    Some(ref mut exploded) => {
                                        exploded.push((row, col));
                                    }
                                }
                            } else {
                                self.reveal_from(idx);
                            }
                        }
                    });

                if let Some(exploded) = exploded {
                    self.status = MinesweeperStatus::Exploded(exploded);
                    return Ok(&self.status);
                }
            } else if adjacent_unrevealed > 0
                && adjacent_unrevealed + adjacent_flagged
                    == self.board[row * self.width + col].adjacent_mine
            {
                self.get_adjacent_cells_by_coords(row, col)
                    .iter_by_idx()
                    .for_each(|idx| {
                        if !self.board[idx].is_flagged && !self.board[idx].is_revealed {
                            self.board[idx].is_flagged = true;
                        }
                    });
            }

            self.status = self.check_game_status();
        }

        Ok(&self.status)
    }

    fn reveal_from(&mut self, idx: usize) {
        if self.board[idx].adjacent_mine != 0 {
            self.board[idx].is_revealed = true;
        } else {
            use std::collections::VecDeque;

            let mut cell_idxs_to_reveal = VecDeque::new();
            cell_idxs_to_reveal.push_back(idx);

            while let Some(cell_idx) = cell_idxs_to_reveal.pop_front() {
                self.board[cell_idx].is_revealed = true;

                for neighbor_idx in self.get_adjacent_cells_by_idx(cell_idx).iter_by_idx() {
                    if !self.board[neighbor_idx].is_flagged
                        && !self.board[neighbor_idx].is_revealed
                    {
                        if self.board[neighbor_idx].adjacent_mine == 0 {
                            cell_idxs_to_reveal.push_back(neighbor_idx);
                        } else {
                            self.board[neighbor_idx].is_revealed = true;
                        }
                    }
                }
            }
        }
    }

    fn check_game_status(&self) -> MinesweeperStatus {
        if self
            .board
            .iter()
            .filter(|cell| !cell.is_mine)
            .all(|cell| cell.is_revealed)
        {
            MinesweeperStatus::Win
        } else {
            MinesweeperStatus::InProgress
        }
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
                .get_adjacent_cells_by_idx(idx)
                .iter_by_idx()
                .filter(|idx| self.board[*idx].is_mine)
                .count();

            self.board[idx].adjacent_mine = count;
        }
    }

    fn get_adjacent_cells_by_coords(&self, row: usize, col: usize) -> MinesweeperAdjacentCells {
        MinesweeperAdjacentCells::new(row, col, self.height, self.width)
    }

    fn get_adjacent_cells_by_idx(&self, idx: usize) -> MinesweeperAdjacentCells {
        self.get_adjacent_cells_by_coords(idx / self.width, idx % self.width)
    }
}

#[derive(Clone, Debug)]
struct MinesweeperAdjacentCells {
    around: [(i128, i128); 8],
    board_height: i128,
    board_width: i128,
    offset: usize,
}

impl Iterator for MinesweeperAdjacentCells {
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

impl MinesweeperAdjacentCells {
    fn new(row: usize, col: usize, board_height: usize, board_width: usize) -> Self {
        let (row, col, board_height, board_width) = (
            row as i128,
            col as i128,
            board_height as i128,
            board_width as i128,
        );

        MinesweeperAdjacentCells {
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
