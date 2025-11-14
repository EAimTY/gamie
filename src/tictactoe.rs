//! Tic-Tac-Toe game implementation
//!
//! A classic 3x3 Tic-Tac-Toe game where two players take turns placing pieces
//! on the board, attempting to get three in a row horizontally, vertically, or diagonally.
//!
//! See [`Game`] for the main game interface.

use core::convert::Infallible;
use thiserror::Error;

const BOARD_WIDTH: usize = 3;
const BOARD_HEIGHT: usize = 3;

/// A Tic-Tac-Toe game instance
///
/// The game is played on a 3x3 board where two players alternate placing their pieces.
/// Player0 always goes first. The game ends when a player gets three pieces in a row
/// (horizontally, vertically, or diagonally) or when the board is full (resulting in a draw).
///
/// # Examples
///
/// ```rust
/// # use gamie::tictactoe::Game;
/// let mut game = Game::new().unwrap();
///
/// // Player0's turn
/// game.put(1, 1).unwrap(); // Center position
///
/// // Player1's turn
/// game.put(0, 0).unwrap(); // Top-left corner
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
/// There are two players in Tic-Tac-Toe. Player0 always makes the first move,
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
/// - [`Win`](Status::Win): A player has won by getting three in a row
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
    /// Creates a new Tic-Tac-Toe game
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
    /// - `row`: The row index (0-2)
    /// - `col`: The column index (0-2)
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than 2)
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
    /// - `row`: The row index (0-2)
    /// - `col`: The column index (0-2)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The position is already occupied ([`Error::PositionOccupied`])
    /// - The game has already ended ([`Error::GameEnded`])
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than 2)
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
        // To determine if the game has ended with the last move, we check
        // all 3 positions in each direction (horizontal, vertical, and diagonal)

        // Check horizontal
        if self.get(last_move.row, 0) == self.get(last_move.row, 1)
            && self.get(last_move.row, 1) == self.get(last_move.row, 2)
        {
            self.status = Status::Win(last_move.player);
            return;
        }

        // Check vertical
        if self.get(0, last_move.col) == self.get(1, last_move.col)
            && self.get(1, last_move.col) == self.get(2, last_move.col)
        {
            self.status = Status::Win(last_move.player);
            return;
        }

        // Check diagonals only if the last move is on a diagonal
        if !((last_move.row == 1) ^ (last_move.col == 1)) {
            // Top-left to bottom-right diagonal
            if self.get(0, 0) == self.get(1, 1) && self.get(1, 1) == self.get(2, 2) {
                self.status = Status::Win(last_move.player);
                return;
            }

            // Top-right to bottom-left diagonal
            if self.get(0, 2) == self.get(1, 1) && self.get(1, 1) == self.get(2, 0) {
                self.status = Status::Win(last_move.player);
                return;
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

#[cfg(test)]
mod tests {
    use crate::tictactoe::*;

    #[test]
    fn test() {
        let mut game = Game::new().unwrap();

        game.put(1, 1).unwrap();

        assert_eq!(game.next_player(), Player::Player1);

        game.put(1, 0).unwrap();

        assert_eq!(game.next_player(), Player::Player0);
        assert!(matches!(game.put(1, 1), Err(Error::PositionOccupied)));

        game.put(2, 2).unwrap();
        game.put(2, 0).unwrap();
        game.put(0, 0).unwrap();

        assert_eq!(game.status(), &Status::Win(Player::Player0));
        assert!(matches!(game.put(0, 2), Err(Error::GameEnded)));
    }
}
