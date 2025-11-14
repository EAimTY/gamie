//! Gomoku (Five in a Row) game implementation
//!
//! Gomoku is a strategy board game played on a 15×15 grid where two players alternate
//! placing pieces. The first player to get exactly five pieces in a row (horizontally,
//! vertically, or diagonally) wins the game.
//!
//! See [`Game`] for the main game interface.

use core::convert::Infallible;
use thiserror::Error;

const BOARD_WIDTH: usize = 15;
const BOARD_HEIGHT: usize = 15;

/// A Gomoku (Five in a Row) game instance
///
/// The game is played on a 15×15 board where two players alternate placing their pieces.
/// Player0 always goes first. The game ends when a player gets exactly five pieces in a row
/// (horizontally, vertically, or diagonally) or when the board is full (resulting in a draw).
///
/// # Examples
///
/// ```rust
/// # use gamie::gomoku::Game;
/// let mut game = Game::new().unwrap();
///
/// // Player0's turn
/// game.put(7, 7).unwrap(); // Center position
///
/// // Player1's turn
/// game.put(8, 8).unwrap();
///
/// // Continue playing...
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    board: [[Option<Player>; BOARD_HEIGHT]; BOARD_WIDTH],
    move_count: usize,
    next_player: Player,
    status: Status,
}

/// Represents a player in the game
///
/// There are two players in Gomoku. Player0 always makes the first move,
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
/// - [`Win`](Status::Win): A player has won by getting five in a row
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
    /// The position is already occupied by a piece
    #[error("position occupied")]
    PositionOccupied,
    /// The game has already ended (either in a win or draw)
    #[error("game ended")]
    GameEnded,
}

struct LastMove {
    player: Player,
    row: usize,
    col: usize,
}

impl Game {
    /// Creates a new Gomoku game
    pub const fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: [[None; BOARD_HEIGHT]; BOARD_WIDTH],
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
    /// - `row`: The row index (0-14)
    /// - `col`: The column index (0-14)
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than or equal to 15)
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        self.board[row][col]
    }

    /// Places a piece for the current player at the specified position
    ///
    /// The current player's piece (returned by [`next_player()`](Self::next_player))
    /// is placed at the given position. After a successful placement, the turn
    /// automatically switches to the other player, and the game status is updated
    /// to check for a win or draw.
    ///
    /// # Parameters
    ///
    /// - `row`: The row index (0-14)
    /// - `col`: The column index (0-14)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The position is already occupied ([`Error::PositionOccupied`])
    /// - The game has already ended ([`Error::GameEnded`])
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than or equal to 15)
    pub fn put(&mut self, row: usize, col: usize) -> Result<(), Error> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(Error::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(Error::PositionOccupied);
        }

        self.board[row][col] = Some(self.next_player);

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
        // To determine if the game ends with the last move, check 9 positions centered at the last move in each direction

        let checking_row_range =
            last_move.row.saturating_sub(4)..=(last_move.row + 4).min(BOARD_HEIGHT - 1);
        let checking_col_range =
            last_move.col.saturating_sub(4)..=(last_move.col + 4).min(BOARD_WIDTH - 1);
        let mut consecutive_pieces = 0;

        // Horizontal
        for col in checking_col_range.clone() {
            if self.get(last_move.row, col) == Some(last_move.player) {
                consecutive_pieces += 1;
                if consecutive_pieces == 5 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // Vertical
        consecutive_pieces = 0;

        for row in checking_row_range.clone() {
            if self.get(row, last_move.col) == Some(last_move.player) {
                consecutive_pieces += 1;
                if consecutive_pieces == 5 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // Top-left to bottom-right diagonal
        consecutive_pieces = 0;

        for (row, col) in checking_row_range.clone().zip(checking_col_range.clone()) {
            if self.get(row, col) == Some(last_move.player) {
                consecutive_pieces += 1;
                if consecutive_pieces == 5 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // Top-right to bottom-left diagonal
        consecutive_pieces = 0;

        for (row, col) in checking_row_range.zip(checking_col_range.rev()) {
            if self.get(row, col) == Some(last_move.player) {
                consecutive_pieces += 1;
                if consecutive_pieces == 5 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // Check for draw
        if self.move_count == BOARD_HEIGHT * BOARD_WIDTH {
            self.status = Status::Draw;
        }
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
