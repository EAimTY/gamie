//! Minesweeper
//!
//! # Examples
//!
//! ```rust
//! # fn minesweeper() {
//! use gamie::minesweeper::Minesweeper;
//! use rand::rngs::ThreadRng;
//!
//! let mut game = Minesweeper::new(&mut ThreadRng::default(), 8, 8, 9).unwrap();
//!
//! game.flag(3, 2).unwrap();
//! // ...
//! game.click(7, 7, true).unwrap();
//! // ...
//! # }
//! ```

extern crate alloc;

use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use core::array::IntoIter;
use rand::Rng;
use snafu::Snafu;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Minesweeper {
    board: Vec<Cell>,
    width: usize,
    height: usize,
    mine_count: usize,
    flag_count: usize,
    status: Status,
}

/// Game status
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    Ongoing,
    Exploded,
    Finished,
}

/// A cell.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cell {
    is_mine: bool,
    status: CellStatus,
}

/// The status of a cell
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CellStatus {
    Hidden,
    Revealed { adjacent_mine_count: usize },
    Flagged,
    Exploded,
}

/// Errors that can occur.
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum MinesweeperError {
    #[snafu(display("too many mines"))]
    TooManyMines,
    #[snafu(display("too many flags"))]
    TooManyFlags,
    #[snafu(display("click on an already flagged cell"))]
    ClickOnFlagged,
    #[snafu(display("click on an already revealed cell"))]
    ClickOnRevealed,
    #[snafu(display("click is doing nothing"))]
    InvalidClick,
    #[snafu(display("game ended"))]
    GameEnded,
}

struct AdjacentCellCoords {
    potentially_adjacent_cell_coords: IntoIter<(usize, usize), 8>,
    board_width: usize,
    board_height: usize,
}

impl Minesweeper {
    /// Create a new Minesweeper game
    pub fn new<R>(
        rng: &mut R,
        width: usize,
        height: usize,
        mut mine_count: usize,
    ) -> Result<Self, MinesweeperError>
    where
        R: Rng,
    {
        if width * height < mine_count {
            return Err(MinesweeperError::TooManyMines);
        }

        let mut board = alloc::vec::Vec::with_capacity(width * height);

        for idx in 0..width * height {
            let is_mine = rng.random_range(0..width * height - idx) < mine_count;

            if is_mine {
                mine_count -= 1;
            }

            board.push(Cell {
                is_mine,
                status: CellStatus::Hidden,
            });
        }

        Ok(Self {
            board,
            width,
            height,
            mine_count,
            flag_count: 0,
            status: Status::Ongoing,
        })
    }

    /// Get the cell on a position
    ///
    /// Panic if the target position is out of bounds
    pub fn get(&self, row: usize, col: usize) -> &Cell {
        assert!(row < self.height);
        assert!(col < self.width);

        &self.board[row * self.width + col]
    }

    /// Click a cell on the game board
    ///
    /// When `auto_flag` is `true`, clicking an already revealed cell will flag its adjacent unflagged-unrevealed cells if the unflagged-revealed cell count around it equals to its adjacent mine count
    ///
    /// Panic when target position out of bounds
    pub fn click(
        &mut self,
        row: usize,
        col: usize,
        auto_flag: bool,
    ) -> Result<(), MinesweeperError> {
        assert!(row < self.height);
        assert!(col < self.width);

        if matches!(self.status, Status::Finished | Status::Exploded) {
            return Err(MinesweeperError::GameEnded);
        }

        match self.board[row * self.width + col].status {
            CellStatus::Hidden => self.reveal([(row, col)]),
            CellStatus::Revealed {
                adjacent_mine_count,
            } => {
                let mut adjacent_flagged_count = 0;
                let mut adjacent_hidden_cell_coords = Vec::new();

                for (row, col) in AdjacentCellCoords::new(self.width, self.height, row, col) {
                    match self.board[row * self.width + col].status {
                        CellStatus::Hidden => adjacent_hidden_cell_coords.push((row, col)),
                        CellStatus::Revealed { .. } => {}
                        CellStatus::Flagged => adjacent_flagged_count += 1,
                        CellStatus::Exploded => unreachable!(),
                    }
                }

                if adjacent_flagged_count == adjacent_mine_count {
                    self.reveal(adjacent_hidden_cell_coords);
                } else if auto_flag
                    && adjacent_hidden_cell_coords.len() + adjacent_flagged_count
                        == adjacent_mine_count
                {
                    for (row, col) in adjacent_hidden_cell_coords {
                        self.board[row * self.width + col].status = CellStatus::Flagged;
                    }
                } else {
                    return Err(MinesweeperError::InvalidClick);
                }
            }
            CellStatus::Flagged => return Err(MinesweeperError::ClickOnFlagged),
            CellStatus::Exploded => unreachable!(),
        }

        self.update_status();

        Ok(())
    }

    /// Flag or unflag a cell on the board
    /// Returns `Err(MinesweeperError::ClickOnRevealed)` if the target cell is already revealed
    ///
    /// Panic when target position out of bounds
    pub fn flag(&mut self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        assert!(row < self.height);
        assert!(col < self.width);

        if matches!(self.status, Status::Finished | Status::Exploded) {
            return Err(MinesweeperError::GameEnded);
        }

        match self.board[row * self.width + col].status {
            CellStatus::Hidden => {
                if self.flag_count == self.mine_count {
                    return Err(MinesweeperError::TooManyFlags);
                }
                self.board[row * self.width + col].status = CellStatus::Flagged;
                self.flag_count += 1;
            }
            CellStatus::Flagged => {
                self.board[row * self.width + col].status = CellStatus::Hidden;
                self.flag_count -= 1;
            }
            CellStatus::Revealed { .. } => return Err(MinesweeperError::ClickOnRevealed),
            CellStatus::Exploded => unreachable!(),
        }

        self.update_status();

        Ok(())
    }

    pub fn mine_count(&self) -> usize {
        self.mine_count
    }

    pub fn flag_count(&self) -> usize {
        self.flag_count
    }

    /// No coords and gamr status validation since this is an internal function
    fn reveal(&mut self, coords: impl Into<VecDeque<(usize, usize)>>) {
        let mut coords = coords.into();
        let mut is_exploded = false;

        while let Some((row, col)) = coords.pop_front() {
            if self.board[row * self.width + col].is_mine {
                self.board[row * self.width + col].status = CellStatus::Exploded;
                self.status = Status::Exploded;
                is_exploded = true;
                continue;
            }

            let adjacent_mine_count = AdjacentCellCoords::new(self.width, self.height, row, col)
                .filter(|(row, col)| self.board[row * self.width + col].is_mine)
                .count();

            self.board[row * self.width + col].status = CellStatus::Revealed {
                adjacent_mine_count,
            };

            if !is_exploded && adjacent_mine_count == 0 {
                coords.extend(AdjacentCellCoords::new(self.width, self.height, row, col));
            }
        }
    }

    fn update_status(&mut self) {
        let all_revealed = self
            .board
            .iter()
            .filter(|cell| !cell.is_mine)
            .all(|cell| matches!(cell.status, CellStatus::Revealed { .. }));

        let all_flagged = self
            .board
            .iter()
            .filter(|cell| cell.is_mine)
            .all(|cell| matches!(cell.status, CellStatus::Flagged));

        if all_revealed || all_flagged {
            self.status = Status::Finished
        }
    }
}

impl Cell {
    pub const fn is_mine(&self) -> bool {
        self.is_mine
    }

    pub const fn status(&self) -> &CellStatus {
        &self.status
    }
}

impl AdjacentCellCoords {
    fn new(board_width: usize, board_height: usize, row: usize, col: usize) -> Self {
        Self {
            potentially_adjacent_cell_coords: [
                (row - 1, col - 1),
                (row - 1, col),
                (row - 1, col + 1),
                (row, col - 1),
                (row, col + 1),
                (row + 1, col - 1),
                (row + 1, col),
                (row + 1, col + 1),
            ]
            .into_iter(),
            board_width,
            board_height,
        }
    }
}

impl Iterator for AdjacentCellCoords {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (row, col) = self.potentially_adjacent_cell_coords.next()?;

            if row < self.board_height && col < self.board_width {
                return Some((row, col));
            }
        }
    }
}
