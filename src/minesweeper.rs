//! Minesweeper game implementation
//!
//! Minesweeper is a puzzle game where players must reveal all cells on a grid that
//! do not contain mines. Cells display the count of adjacent mines, and players can
//! flag cells they believe contain mines. Clicking on a mine ends the game.
//!
//! See [`Game`] for the main game interface.
//!
//! # Examples
//!
//! ```rust
//! # fn minesweeper() {
//! use gamie::minesweeper::Game;
//!
//! let mut game = Game::new(&mut rand::rng(), 8, 8, 9).unwrap();
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
use thiserror::Error;

/// A Minesweeper game instance
///
/// This struct represents a complete Minesweeper game with a board of cells,
/// some of which contain mines. The goal is to reveal all non-mine cells
/// without clicking on any mines.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    board: Vec<Cell>,
    width: usize,
    height: usize,
    mine_count: usize,
    flag_count: usize,
    status: Status,
}

/// The current status of the game
///
/// The game can be in one of three states:
/// - [`Ongoing`](Status::Ongoing): The game is still in progress
/// - [`Exploded`](Status::Exploded): A mine was clicked and the game is lost
/// - [`Finished`](Status::Finished): All non-mine cells are revealed or all mines are flagged
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    /// The game is still in progress
    Ongoing,
    /// The player clicked on a mine and lost
    Exploded,
    /// The player successfully revealed all non-mine cells or flagged all mines
    Finished,
}

/// A single cell on the game board
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cell {
    is_mine: bool,
    status: CellStatus,
}

/// The visibility and interaction status of a cell
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CellStatus {
    /// The cell has not been revealed or flagged yet
    Hidden,
    /// The cell has been revealed and shows the count of adjacent mines
    Revealed { adjacent_mine_count: usize },
    /// The cell has been flagged by the player as potentially containing a mine
    Flagged,
    /// The cell contained a mine and was clicked, ending the game
    Exploded,
}

/// Errors that can occur when placing a piece on the board
///
/// These errors prevent invalid moves from being made during the game.
#[derive(Debug, Error)]
pub enum Error {
    /// Attempted to create a board with more mines than available cells
    #[error("too many mines")]
    TooManyMines,
    /// Attempted to place more flags than there are mines
    #[error("too many flags")]
    TooManyFlags,
    /// Attempted to click on a cell that is already flagged
    #[error("attempted to click a flagged cell")]
    ClickOnFlagged,
    /// Attempted to click on a cell that is already revealed
    #[error("attempted to click an already revealed cell")]
    ClickOnRevealed,
    /// The click action would have no effect
    #[error("click has no effect")]
    InvalidClick,
    /// Attempted to perform an action after the game has already ended
    #[error("game has ended")]
    GameEnded,
}

/// Iterator that yields the coordinates of all valid adjacent cells
///
/// This iterator handles boundary checking and wrapping for cells at the edges of the board
struct AdjacentCellCoords {
    potentially_adjacent_cell_coords: IntoIter<(usize, usize), 8>,
    board_width: usize,
    board_height: usize,
}

impl Game {
    /// Creates a new Minesweeper game with the specified dimensions and mine count
    ///
    /// # Parameters
    ///
    /// - `rng`: Random number generator used to place mines randomly on the board
    /// - `width`: The number of columns in the game board
    /// - `height`: The number of rows in the game board
    /// - `mine_count`: The number of mines to place on the board
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `mine_count` is greater than the total number of cells ([`Error::TooManyMines`])
    pub fn new<R>(
        rng: &mut R,
        width: usize,
        height: usize,
        mut mine_count: usize,
    ) -> Result<Self, Error>
    where
        R: Rng,
    {
        if width * height < mine_count {
            return Err(Error::TooManyMines);
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

    /// Gets a reference to the cell at the specified position
    ///
    /// # Parameters
    ///
    /// - `row`: The row index (0-based, top to bottom)
    /// - `col`: The column index (0-based, left to right)
    ///
    /// # Panics
    ///
    /// Panics if `row >= height` or `col >= width`
    pub fn get(&self, row: usize, col: usize) -> &Cell {
        assert!(row < self.height);
        assert!(col < self.width);

        &self.board[row * self.width + col]
    }

    /// Clicks a cell on the game board to reveal it
    ///
    /// This is the primary interaction method. When clicking a hidden cell:
    /// - If it's a mine, the game ends with status `Exploded`
    /// - If it's not a mine, it reveals the cell and shows the count of adjacent mines
    /// - If there are no adjacent mines, automatically reveals all adjacent cells recursively
    ///
    /// When `auto_flag` is `true`, clicking an already revealed cell will:
    /// - Automatically flag adjacent hidden cells if the number of hidden and flagged cells
    ///   equals the adjacent mine count (acts as a quick-flag shortcut)
    /// - Reveal adjacent hidden cells if the number of flagged cells equals the adjacent
    ///   mine count (acts as a chord/quick-reveal shortcut)
    ///
    /// # Parameters
    ///
    /// - `row`: The row index of the cell to click
    /// - `col`: The column index of the cell to click
    /// - `auto_flag`: Whether to enable automatic flagging/revealing on already revealed cells
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The game has already ended ([`Error::GameEnded`])
    /// - The cell is flagged ([`Error::ClickOnFlagged`])
    /// - The click on a revealed cell has no effect ([`Error::InvalidClick`])
    ///
    /// # Panics
    ///
    /// Panics if `row >= height` or `col >= width`
    pub fn click(&mut self, row: usize, col: usize, auto_flag: bool) -> Result<(), Error> {
        assert!(row < self.height);
        assert!(col < self.width);

        if matches!(self.status, Status::Finished | Status::Exploded) {
            return Err(Error::GameEnded);
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
                    return Err(Error::InvalidClick);
                }
            }
            CellStatus::Flagged => return Err(Error::ClickOnFlagged),
            CellStatus::Exploded => unreachable!(),
        }

        self.update_status();

        Ok(())
    }

    /// Toggles a flag on a cell to mark it as potentially containing a mine
    ///
    /// Flagging is used to mark cells you believe contain mines without revealing them.
    /// - If the cell is currently hidden, it will be flagged
    /// - If the cell is already flagged, it will be unflagged (returned to hidden state)
    /// - Cannot flag more cells than the total mine count
    ///
    /// # Parameters
    ///
    /// - `row`: The row index of the cell to flag/unflag
    /// - `col`: The column index of the cell to flag/unflag
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The game has already ended ([`Error::GameEnded`])
    /// - Attempting to place more flags than there are mines ([`Error::TooManyFlags`])
    /// - The cell is already revealed ([`Error::ClickOnRevealed`])
    ///
    /// # Panics
    ///
    /// Panics if `row >= height` or `col >= width`
    pub fn flag(&mut self, row: usize, col: usize) -> Result<(), Error> {
        assert!(row < self.height);
        assert!(col < self.width);

        if matches!(self.status, Status::Finished | Status::Exploded) {
            return Err(Error::GameEnded);
        }

        match self.board[row * self.width + col].status {
            CellStatus::Hidden => {
                if self.flag_count == self.mine_count {
                    return Err(Error::TooManyFlags);
                }
                self.board[row * self.width + col].status = CellStatus::Flagged;
                self.flag_count += 1;
            }
            CellStatus::Flagged => {
                self.board[row * self.width + col].status = CellStatus::Hidden;
                self.flag_count -= 1;
            }
            CellStatus::Revealed { .. } => return Err(Error::ClickOnRevealed),
            CellStatus::Exploded => unreachable!(),
        }

        self.update_status();

        Ok(())
    }

    /// Gets the total number of mines on the board
    pub const fn mine_count(&self) -> usize {
        self.mine_count
    }

    /// Gets the current number of flags placed
    pub const fn flag_count(&self) -> usize {
        self.flag_count
    }

    /// Gets the current game status
    ///
    /// Returns whether the game is ongoing, exploded, or finished.
    /// The status is automatically updated after each move.
    pub const fn status(&self) -> &Status {
        &self.status
    }

    /// Internal function that reveals cells without validating coordinates or game status
    ///
    /// This method performs a breadth-first search to reveal cells:
    /// - If a cell contains a mine, it marks it as exploded and sets game status to `Exploded`
    /// - If a cell has no adjacent mines, it recursively reveals all adjacent cells
    /// - Otherwise, it reveals the cell showing the adjacent mine count
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

    /// Updates the game status based on current board state
    ///
    /// The game is considered finished when either:
    /// - All non-mine cells have been revealed, OR
    /// - All mine cells have been flagged
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
    /// Checks if this cell contains a mine
    pub const fn is_mine(&self) -> bool {
        self.is_mine
    }

    /// Gets the current status of this cell
    pub const fn status(&self) -> &CellStatus {
        &self.status
    }
}

impl AdjacentCellCoords {
    /// Creates a new iterator for adjacent cell coordinates
    ///
    /// Returns an iterator that will yield coordinates of all valid cells
    /// adjacent to the given position (up to 8 cells in all directions)
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
