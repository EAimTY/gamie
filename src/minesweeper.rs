//! Minesweeper
//!
//! Check struct [`Minesweeper`](https://docs.rs/gamie/*/gamie/minesweeper/struct.Minesweeper.html) for more information
//!
//! # Examples
//!
//! ```rust
//! # fn minesweeper() {
//! use gamie::minesweeper::Minesweeper;
//! use rand::rngs::ThreadRng;
//!
//! let mut game = Minesweeper::new(8, 8, 9, ThreadRng::default()).unwrap();
//!
//! game.toggle_flag(3, 2).unwrap();
//! // ...
//! game.click(7, 7, true).unwrap();
//! // ...
//! # }
//! ```

use crate::std_lib::{iter, Vec, VecDeque};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use rand::{
    distributions::{Distribution, Uniform},
    Rng,
};
use snafu::Snafu;

/// Minesweeper
///
/// To avoid unessecary memory allocation, the game board is stored in a single `Vec` rather than a nested one.
///
/// Passing an invalid position to a method will cause panic. Check the target position validity first when dealing with user input
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Minesweeper<R> {
    board: Vec<Cell>,
    height: usize,
    width: usize,
    mine: usize,
    rng: R,
    step_count: usize,
    flag_count: usize,
    status: Status,
}

/// The cell in the board.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cell {
    pub is_mine: bool,
    pub mine_adjacent: usize,
    pub is_revealed: bool,
    pub is_flagged: bool,
}

impl Cell {
    fn new(is_mine: bool) -> Self {
        Self {
            is_mine,
            mine_adjacent: 0,
            is_revealed: false,
            is_flagged: false,
        }
    }
}

/// Game status
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Status {
    Win,
    Exploded(Vec<(usize, usize)>),
    InProgress,
}

impl<R: Rng> Minesweeper<R> {
    /// Create a new Minesweeper game
    ///
    /// A random number generator is required for randomizing mine positions
    ///
    /// Return `Err(MinesweeperError::TooManyMines)` if `(height - 1) * (width - 1) < mines`
    ///
    /// # Examples
    /// ```rust
    /// # fn minesweeper() {
    /// use gamie::minesweeper::Minesweeper;
    /// use rand::rngs::ThreadRng;
    ///
    /// let mut game = Minesweeper::new(8, 8, 9, ThreadRng::default()).unwrap();
    /// # }
    /// ```
    pub fn new(
        height: usize,
        width: usize,
        mines: usize,
        rng: R,
    ) -> Result<Self, MinesweeperError> {
        if (height - 1) * (width - 1) < mines {
            return Err(MinesweeperError::TooManyMines);
        }

        let board = iter::repeat(Cell::new(true))
            .take(mines)
            .chain(iter::repeat(Cell::new(false)).take(height * width - mines))
            .collect();

        let mut minesweeper = Self {
            board,
            height,
            width,
            mine: mines,
            rng,
            step_count: 0,
            flag_count: 0,
            status: Status::InProgress,
        };

        minesweeper.randomize();

        Ok(minesweeper)
    }

    /// Get a cell reference from the game board
    /// Panic when target position out of bounds
    pub fn get(&self, row: usize, col: usize) -> &Cell {
        assert!(row < self.height);
        assert!(col < self.width);

        &self.board[row * self.width + col]
    }

    /// Click a cell on the game board
    ///
    /// The first click is always a safe click
    ///
    /// Clicking an already revealed cell will unreveal its adjacent cells if the flagged cell count around it equals to its adjacent mine count
    /// When `auto_flag` is `true`, clicking an already revealed cell will flag its adjacent unflagged-unrevealed cells if the unflagged-revealed cell count around it equals to its adjacent mine count
    ///
    /// The return value indicates if the game board is changed from the click
    ///
    /// Panic when target position out of bounds
    pub fn click(
        &mut self,
        row: usize,
        col: usize,
        auto_flag: bool,
    ) -> Result<bool, MinesweeperError> {
        assert!(row < self.height);
        assert!(col < self.width);

        if self.is_ended() {
            return Err(MinesweeperError::GameEnded);
        }

        if !self.board[row * self.width + col].is_revealed {
            if self.step_count == 0 {
                while self.board[row * self.width + col].is_mine
                    || self.board[row * self.width + col].mine_adjacent > 0
                {
                    self.randomize();
                }
            }

            self.click_unrevealed(row, col)?;
            self.step_count += 1;
            Ok(true)
        } else if self.click_revealed(row, col, auto_flag)? {
            self.step_count += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Flag or unflag a cell on the board
    /// Return Err(MinesweeperError::AlreadyRevealed) if the target cell is already revealed
    ///
    /// Panic when target position out of bounds
    pub fn toggle_flag(&mut self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        assert!(row < self.height);
        assert!(col < self.width);

        if self.is_ended() {
            return Err(MinesweeperError::GameEnded);
        }

        if self.board[row * self.width + col].is_revealed {
            return Err(MinesweeperError::AlreadyRevealed);
        }

        if !self.board[row * self.width + col].is_flagged {
            if self.flag_count == self.mine {
                return Err(MinesweeperError::TooManyFlags);
            }

            self.flag_count += 1;
        } else {
            self.flag_count -= 1;
        }

        self.board[row * self.width + col].is_flagged =
            !self.board[row * self.width + col].is_flagged;

        self.check_game_status();

        Ok(())
    }

    /// Check if the game was end
    pub fn is_ended(&self) -> bool {
        self.status != Status::InProgress
    }

    /// Get the game status
    pub fn get_game_status(&self) -> &Status {
        &self.status
    }

    /// Get the height of the game board
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Get the width of the game board
    pub fn get_width(&self) -> usize {
        self.width
    }

    /// Get the number of mines in the game board
    pub fn get_mine_count(&self) -> usize {
        self.mine
    }

    /// Get the number of flags used
    pub fn get_flag_count(&self) -> usize {
        self.flag_count
    }

    /// Get the number of steps taken
    pub fn get_step_count(&self) -> usize {
        self.step_count
    }

    fn randomize(&mut self) {
        let range = Uniform::from(0..self.height * self.width);

        for idx in 0..self.height * self.width {
            self.board.swap(idx, range.sample(&mut self.rng));
        }

        self.update_adjacent_mine_count();
    }

    fn click_unrevealed(&mut self, row: usize, col: usize) -> Result<(), MinesweeperError> {
        if self.board[row * self.width + col].is_flagged {
            return Err(MinesweeperError::AlreadyFlagged);
        }

        if self.board[row * self.width + col].is_mine {
            self.status = Status::Exploded(vec![(row, col)]);
            return Ok(());
        }

        self.reveal_from(row * self.width + col);
        self.check_game_status();

        Ok(())
    }

    fn click_revealed(
        &mut self,
        row: usize,
        col: usize,
        auto_flag: bool,
    ) -> Result<bool, MinesweeperError> {
        let mut is_changed = false;

        if self.board[row * self.width + col].mine_adjacent > 0 {
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
                if adjacent_flagged == self.board[row * self.width + col].mine_adjacent {
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
                        self.status = Status::Exploded(exploded);
                        return Ok(true);
                    }
                }

                if auto_flag
                    && adjacent_unrevealed + adjacent_flagged
                        == self.board[row * self.width + col].mine_adjacent
                {
                    self.get_adjacent_cells(row, col).for_each(|idx| {
                        if !self.board[idx].is_flagged && !self.board[idx].is_revealed {
                            self.flag_count += 1;
                            self.board[idx].is_flagged = true;
                            is_changed = true;
                        }
                    });
                }
            }

            self.check_game_status();
        }

        Ok(is_changed)
    }

    fn reveal_from(&mut self, idx: usize) {
        if self.board[idx].mine_adjacent != 0 {
            self.board[idx].is_revealed = true;
        } else {
            let mut cell_idxs_to_reveal = VecDeque::new();
            cell_idxs_to_reveal.push_back(idx);

            while let Some(cell_idx) = cell_idxs_to_reveal.pop_front() {
                self.board[cell_idx].is_revealed = true;

                for neighbor_idx in
                    self.get_adjacent_cells(cell_idx / self.width, cell_idx % self.width)
                {
                    if !self.board[neighbor_idx].is_flagged && !self.board[neighbor_idx].is_revealed
                    {
                        if self.board[neighbor_idx].mine_adjacent == 0 {
                            cell_idxs_to_reveal.push_back(neighbor_idx);
                        } else {
                            self.board[neighbor_idx].is_revealed = true;
                        }
                    }
                }
            }
        }
    }

    fn check_game_status(&mut self) {
        let all_revealed = self
            .board
            .iter()
            .filter(|cell| !cell.is_mine)
            .all(|cell| cell.is_revealed);

        let all_flagged = self
            .board
            .iter()
            .filter(|cell| cell.is_mine)
            .all(|cell| cell.is_flagged);

        self.status = if all_revealed || all_flagged {
            Status::Win
        } else {
            Status::InProgress
        };
    }

    fn update_adjacent_mine_count(&mut self) {
        for idx in 0..self.height * self.width {
            let count = self
                .get_adjacent_cells(idx / self.width, idx % self.width)
                .filter(|idx| self.board[*idx].is_mine)
                .count();

            self.board[idx].mine_adjacent = count;
        }
    }

    fn get_adjacent_cells(&self, row: usize, col: usize) -> AdjacentCells {
        AdjacentCells::new(row, col, self.height, self.width)
    }
}

#[derive(Clone)]
struct AdjacentCells {
    around: [(isize, isize); 8],
    board_height: isize,
    board_width: isize,
    offset: usize,
}

impl Iterator for AdjacentCells {
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

impl AdjacentCells {
    fn new(row: usize, col: usize, board_height: usize, board_width: usize) -> Self {
        let (row, col, board_height, board_width) = (
            row as isize,
            col as isize,
            board_height as isize,
            board_width as isize,
        );

        AdjacentCells {
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

/// Errors that can occur.
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum MinesweeperError {
    #[snafu(display("Too many mines"))]
    TooManyMines,
    #[snafu(display("Too many flags"))]
    TooManyFlags,
    #[snafu(display("Clicked an already flagged cell"))]
    AlreadyFlagged,
    #[snafu(display("Clicked an already revealed cell"))]
    AlreadyRevealed,
    #[snafu(display("The game was already end"))]
    GameEnded,
}
