//! Connect Four game implementation
//!
//! Connect Four is a two-player strategy game played on a 7×6 vertical grid.
//! Players take turns dropping pieces into columns, with pieces falling to the
//! lowest available position. The first player to get four pieces in a row
//! (horizontally, vertically, or diagonally) wins.
//!
//! See [`Game`] for the main game interface.

use core::{
    convert::Infallible,
    fmt::{Debug, Error as FmtError, Formatter},
};
use thiserror::Error;

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

/// A Connect Four game instance
///
/// The game is played on a 7×6 vertical board where two players alternate dropping
/// pieces into columns. Player0 always goes first. The game ends when a player gets
/// four pieces in a row (horizontally, vertically, or diagonally) or when the board
/// is full (resulting in a draw).
///
/// # Examples
///
/// ```rust
/// # use gamie::connect_four::Game;
/// let mut game = Game::new().unwrap();
///
/// // Player0's turn - drop piece in column 3
/// game.put(3).unwrap();
///
/// // Player1's turn - drop piece in column 2
/// game.put(2).unwrap();
///
/// // Continue playing...
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    columns: [Column; BOARD_WIDTH],
    move_count: usize,
    next_player: Player,
    status: Status,
}

/// Represents a player in the game
///
/// There are two players in Connect Four. Player0 always makes the first move,
/// followed by Player1, and they continue alternating turns throughout the game.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    Player0,
    Player1,
}

/// The current status of the game
///
/// The game can be in one of three states:
/// - [`Ongoing`](Status::Ongoing): The game is still in progress
/// - [`Win`](Status::Win): A player has won by getting four in a row
/// - [`Draw`](Status::Draw): All positions are filled with no winner
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    Ongoing,
    Draw,
    Win(Player),
}

/// Errors that can occur when placing a piece on the board
///
/// These errors prevent invalid moves from being made during the game.
#[derive(Debug, Error)]
pub enum Error {
    /// The specified column is already full
    #[error("column filled")]
    ColumnFilled,
    /// The game has already ended (either in a win or draw)
    #[error("game ended")]
    GameEnded,
}

/// A column in the board
///
/// Since pieces are placed from the bottom, the column is represented as a grow-only stack.
/// `Option<Player>` is not needed since the number of filled cells is tracked separately.
#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Column {
    cells: [Player; BOARD_HEIGHT],
    filled: usize,
}

struct LastMove {
    player: Player,
    row: usize,
    col: usize,
}

impl Game {
    /// Creates a new Connect Four game
    pub const fn new() -> Result<Self, Infallible> {
        Ok(Self {
            columns: [Column {
                cells: [Player::Player0; BOARD_HEIGHT],
                filled: 0,
            }; BOARD_WIDTH],
            move_count: 0,
            next_player: Player::Player0,
            status: Status::Ongoing,
        })
    }

    /// Gets the piece at the specified position
    ///
    /// Returns `Some(Player)` if a piece is present at the given position,
    /// or `None` if the position is empty.
    ///
    /// # Parameters
    ///
    /// - `row`: The row index (0-5)
    /// - `col`: The column index (0-6)
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        let column = &self.columns[col];

        if row >= BOARD_HEIGHT - column.filled {
            Some(column.cells[row])
        } else {
            None
        }
    }

    /// Places a piece for the current player in the specified column
    ///
    /// The current player's piece (returned by [`next_player()`](Self::next_player))
    /// is placed in the specified column. The piece drops to the lowest available row
    /// in that column. After a successful placement, the turn automatically switches
    /// to the other player, and the game status is updated to check for a win or draw.
    ///
    /// # Parameters
    ///
    /// - `col`: The column index (0-6)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The column is already full ([`Error::ColumnFilled`])
    /// - The game has already ended ([`Error::GameEnded`])
    ///
    /// # Panics
    ///
    /// Panics if `col` is out of bounds (greater than 6)
    pub fn put(&mut self, col: usize) -> Result<(), Error> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(Error::GameEnded);
        }

        if self.columns[col].filled == BOARD_HEIGHT {
            return Err(Error::ColumnFilled);
        }

        let column = &mut self.columns[col];

        let row = BOARD_HEIGHT - 1 - column.filled;

        column.cells[row] = self.next_player;
        column.filled += 1;

        let last_move = LastMove {
            player: self.next_player,
            row,
            col,
        };

        self.move_count += 1;
        self.next_player = self.next_player.other();

        self.update_status(last_move);

        Ok(())
    }

    /// Gets the next player whose turn it is to move
    ///
    /// Returns the player who should make the next move. This changes
    /// after each successful call to [`put()`](Self::put).
    pub const fn next_player(&self) -> Player {
        self.next_player
    }

    /// Gets the current game status
    ///
    /// Returns whether the game is ongoing, ended in a draw, or won by a player.
    /// The status is automatically updated after each move.
    pub const fn status(&self) -> &Status {
        &self.status
    }

    fn update_status(&mut self, last_move: LastMove) {
        const DIRECTIONS: [(isize, isize); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];

        for (row_delta, col_delta) in DIRECTIONS {
            let connected = 1
                + self.count_direction(&last_move, row_delta, col_delta)
                + self.count_direction(&last_move, -row_delta, -col_delta);

            if connected >= 4 {
                self.status = Status::Win(last_move.player);
                return;
            }
        }

        // Check if the game is a draw
        if self.move_count == BOARD_HEIGHT * BOARD_WIDTH {
            self.status = Status::Draw;
        }
    }

    fn count_direction(&self, last_move: &LastMove, row_delta: isize, col_delta: isize) -> usize {
        let mut row = last_move.row as isize + row_delta;
        let mut col = last_move.col as isize + col_delta;
        let mut count = 0;

        while row >= 0 && row < BOARD_HEIGHT as isize && col >= 0 && col < BOARD_WIDTH as isize {
            if self.get(row as usize, col as usize) != Some(last_move.player) {
                break;
            }

            count += 1;
            row += row_delta;
            col += col_delta;
        }

        count
    }
}

impl Player {
    /// Returns the opposite player
    ///
    /// If this player is Player0, returns Player1, and vice versa.
    pub const fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let mut board = [[None; BOARD_HEIGHT]; BOARD_WIDTH];

        for (col, board_column) in board.iter_mut().enumerate() {
            let column = &self.columns[col];

            for (row, cell) in board_column
                .iter_mut()
                .enumerate()
                .skip(BOARD_HEIGHT - column.filled)
            {
                *cell = Some(column.cells[row]);
            }
        }

        f.debug_struct("Game")
            .field("board", &board)
            .field("move_count", &self.move_count)
            .field("next_player", &self.next_player)
            .field("status", &self.status)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::connect_four::*;
    use core::fmt::Write as _;

    #[test]
    fn test() {
        let mut game = Game::new().unwrap();

        game.put(3).unwrap();
        game.put(2).unwrap();
        game.put(2).unwrap();
        game.put(1).unwrap();
        game.put(1).unwrap();
        game.put(0).unwrap();
        game.put(3).unwrap();
        game.put(0).unwrap();
        game.put(1).unwrap();
        game.put(6).unwrap();
        game.put(2).unwrap();
        game.put(6).unwrap();
        game.put(3).unwrap();
        game.put(5).unwrap();
        game.put(0).unwrap();

        assert_eq!(game.status(), &Status::Win(Player::Player0));
    }

    #[test]
    fn detects_diagonal_win_near_board_edge() {
        let mut game = Game::new().unwrap();

        for col in [
            6, 6, 6, 6, 6, 6, 5, 5, 5, 4, 5, 5, 5, 4, 3, 4, 4, 0, 4, 4, 3, 3, 3, 3, 3,
        ] {
            game.put(col).unwrap();
        }

        assert_eq!(game.get(0, 3), Some(Player::Player0));
        assert_eq!(game.get(1, 4), Some(Player::Player0));
        assert_eq!(game.get(2, 5), Some(Player::Player0));
        assert_eq!(game.get(3, 6), Some(Player::Player0));
        assert_eq!(game.status(), &Status::Win(Player::Player0));
    }

    #[test]
    fn debug_board_uses_actual_row_positions() {
        let mut game = Game::new().unwrap();

        game.put(3).unwrap();
        game.put(2).unwrap();

        extern crate alloc;
        let mut debug = alloc::string::String::new();
        write!(debug, "{game:?}").unwrap();

        assert!(debug.contains("[None, None, None, None, None, Some(Player1)]"));
        assert!(debug.contains("[None, None, None, None, None, Some(Player0)]"));
        assert!(!debug.contains("[Some(Player0), None, None, None, None, None]"));
    }
}
