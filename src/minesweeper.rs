//! The Minesweeper game.
//!
//! # Examples
//!
//! ```rust
//! # fn minesweeper() {
//! use gamie::minesweeper::*;
//!
//! let mut game = Minesweeper::new(8, 8, 9).unwrap();
//! game.click(7, 7, true).unwrap();
//! // ...
//! game.toggle_flag(3, 2).unwrap();
//! // ...
//! # }
//! ```

/// The Minesweeper game.
///
/// To avoid unessecary memory allocation, the game board is stored in a single `Vec` rather than a nested one. Use the `get` method to access the board instead of using the `board` field directly.
///
/// If you pass an invalid position to a method, the game will panic. Make sure to use `check_position_validity` to check the target position when dealing with user input.
#[derive(Clone, Debug)]
pub struct Minesweeper {
    pub board: Vec<MinesweeperCell>,
    pub height: usize,
    pub width: usize,
    pub state: MinesweeperState,
}

/// The cell of the Minesweeper board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MinesweeperCell {
    pub is_mine: bool,
    pub adjacent_mine: usize,
    pub is_revealed: bool,
    pub is_flagged: bool,
}

impl MinesweeperCell {
    fn new(is_mine: bool) -> Self {
        Self {
            is_mine,
            adjacent_mine: 0,
            is_revealed: false,
            is_flagged: false,
        }
    }
}

/// The Minesweeper game state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MinesweeperState {
    Win,
    Exploded(Vec<(usize, usize)>),
    InProgress,
}

impl Minesweeper {
    /// Create a new Minesweeper game. Return `Err(MinesweeperError::TooManyMines)` if `height * width < mines`.
    pub fn new(height: usize, width: usize, mines: usize) -> Result<Self, MinesweeperError> {
        if height * width < mines {
            return Err(MinesweeperError::TooManyMines);
        }

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
            state: MinesweeperState::InProgress,
        };
        minesweeper.randomize();

        Ok(minesweeper)
    }

    /// Randomize the Minesweeper board.
    /// Useful if the first click is on a mine.
    pub fn randomize(&mut self) {
        use rand::distributions::{Distribution, Uniform};

        let mut rng = rand::thread_rng();
        let range = Uniform::from(0..self.height * self.width);

        for idx in 0..self.height * self.width {
            self.board.swap(idx, range.sample(&mut rng));
        }

        self.update_around_mine_count();
    }

    /// Check if the game was already end.
    pub fn is_ended(&self) -> bool {
        self.state != MinesweeperState::InProgress
    }

    /// Get the state of the game.
    pub fn state(&self) -> &MinesweeperState {
        &self.state
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &MinesweeperCell {
        &self.board[row * self.width + col]
    }

    /// Get a mutable cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut MinesweeperCell {
        &mut self.board[row * self.width + col]
    }

    /// Click a cell on the game board.
    ///
    /// Clicking an already revealed cell will unreveal its adjacent cells if the flagged cell count around it equals to its adjacent mine count.
    /// When `auto_flag` is `true`, clicking an already revealed cell will flag its adjacent unflagged-unrevealed cells if the unflagged-revealed cell count around it equals to its adjacent mine count.
    ///
    /// The `bool` in the return value indicates if the game board is changed from the click.
    ///
    /// Panic if the target position is out of bounds.
    pub fn click(
        &mut self,
        row: usize,
        col: usize,
        auto_flag: bool,
    ) -> Result<bool, MinesweeperError> {
        if !self.board[row * self.width + col].is_revealed {
            self.click_unrevealed(row, col)?;
            Ok(true)
        } else {
            Ok(self.click_revealed(row, col, auto_flag)?)
        }
    }

    /// Flag or unflag a cell on the Minesweeper board.
    /// Return Err(MinesweeperError::AlreadyRevealed) if the target cell is already revealed.
    ///
    /// Panic if the target position is out of bounds.
    pub fn toggle_flag(&mut self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        if self.board[row * self.width + col].is_revealed {
            return Err(MinesweeperError::AlreadyRevealed);
        }

        self.board[row * self.width + col].is_flagged =
            !self.board[row * self.width + col].is_flagged;

        self.check_game_state();

        Ok(())
    }

    /// Check if the game was already end and the target position is in the board.
    pub fn check_position_validity(&self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        if self.state != MinesweeperState::InProgress {
            return Err(MinesweeperError::GameEnded);
        }

        if row >= self.height || col >= self.width {
            return Err(MinesweeperError::OutOfBounds);
        }

        Ok(())
    }

    fn click_unrevealed(&mut self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        if self.board[row * self.width + col].is_flagged {
            return Err(MinesweeperError::AlreadyFlagged);
        }

        if self.board[row * self.width + col].is_mine {
            self.state = MinesweeperState::Exploded(vec![(row, col)]);
            return Ok(());
        }

        self.reveal_from(row * self.width + col);
        self.check_game_state();

        Ok(())
    }

    fn click_revealed(
        &mut self,
        row: usize,
        col: usize,
        auto_flag: bool,
    ) -> Result<bool, MinesweeperError> {
        let mut is_changed = false;

        if self.board[row * self.width + col].adjacent_mine > 0 {
            let mut adjacent_all = 0;
            let mut adjacent_revealed = 0;
            let mut adjacent_flagged = 0;

            self.get_adjacent_cells(row, col)
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

            if adjacent_unrevealed > 0 {
                if adjacent_flagged == self.board[row * self.width + col].adjacent_mine {
                    let mut exploded = None;

                    self.get_adjacent_cells(row, col).for_each(|idx| {
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
                                is_changed = true;
                            }
                        }
                    });

                    if let Some(exploded) = exploded {
                        self.state = MinesweeperState::Exploded(exploded);
                        return Ok(true);
                    }
                }

                if auto_flag
                    && adjacent_unrevealed + adjacent_flagged
                        == self.board[row * self.width + col].adjacent_mine
                {
                    self.get_adjacent_cells(row, col).for_each(|idx| {
                        if !self.board[idx].is_flagged && !self.board[idx].is_revealed {
                            self.board[idx].is_flagged = true;
                            is_changed = true;
                        }
                    });
                }
            }

            self.check_game_state();
        }

        Ok(is_changed)
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

                for neighbor_idx in
                    self.get_adjacent_cells(cell_idx / self.width, cell_idx % self.width)
                {
                    if !self.board[neighbor_idx].is_flagged && !self.board[neighbor_idx].is_revealed
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

    fn check_game_state(&mut self) {
        self.state = if self
            .board
            .iter()
            .filter(|cell| !cell.is_mine)
            .all(|cell| cell.is_revealed)
        {
            MinesweeperState::Win
        } else {
            MinesweeperState::InProgress
        };
    }

    fn update_around_mine_count(&mut self) {
        for idx in 0..self.height * self.width {
            let count = self
                .get_adjacent_cells(idx / self.width, idx % self.width)
                .filter(|idx| self.board[*idx].is_mine)
                .count();

            self.board[idx].adjacent_mine = count;
        }
    }

    fn get_adjacent_cells(&self, row: usize, col: usize) -> MinesweeperAdjacentCells {
        MinesweeperAdjacentCells::new(row, col, self.height, self.width)
    }
}

#[derive(Clone)]
struct MinesweeperAdjacentCells {
    around: [(i128, i128); 8],
    board_height: i128,
    board_width: i128,
    offset: usize,
}

impl Iterator for MinesweeperAdjacentCells {
    type Item = usize;

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
                (row * self.board_width + col) as usize
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
}

use thiserror::Error;

/// Errors that can occur.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum MinesweeperError {
    #[error("Too many mines")]
    TooManyMines,
    #[error("Position out of bounds")]
    OutOfBounds,
    #[error("Clicked an already flagged cell")]
    AlreadyFlagged,
    #[error("Clicked an already revealed cell")]
    AlreadyRevealed,
    #[error("The game was already ended")]
    GameEnded,
}
