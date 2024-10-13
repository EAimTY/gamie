//! Tic-Tac-Toe
//!
//! Check struct [`TicTacToe`] for more information

use core::convert::Infallible;
use snafu::Snafu;

const BOARD_WIDTH: usize = 3;
const BOARD_HEIGHT: usize = 3;

/// Tic-Tac-Toe
///
/// # Examples
///
/// ```rust
/// # use gamie::tictactoe::TicTacToe;
/// let mut game = TicTacToe::new().unwrap();
/// game.put(1, 0).unwrap();
/// game.put(0, 1).unwrap();
/// // ...
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TicTacToe {
    board: [[Option<Player>; BOARD_HEIGHT]; BOARD_WIDTH],
    move_count: usize,
    next_player: Player,
    status: Status,
}

/// Player
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Player {
    Player0,
    Player1,
}

/// Game status
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Status {
    Ongoing,
    Draw,
    Win(Player),
}

/// Errors that can occur when placing a piece onto the board
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum TicTacToeError {
    #[snafu(display("position occupied"))]
    PositionOccupied,
    #[snafu(display("game ended"))]
    GameEnded,
}

struct LastMove {
    player: Player,
    row: usize,
    col: usize,
}

impl TicTacToe {
    /// Create a new Tic-Tac-Toe game
    pub const fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: [[None; BOARD_HEIGHT]; BOARD_WIDTH],
            move_count: 0,
            next_player: Player::Player0,
            status: Status::Ongoing,
        })
    }

    /// Get a piece at a position
    ///
    /// Panic if the target position is out of bounds
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        self.board[row][col]
    }

    /// Put a piece
    ///
    /// Panic if the target position is out of bounds
    pub fn put(&mut self, row: usize, col: usize) -> Result<(), TicTacToeError> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(TicTacToeError::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(TicTacToeError::PositionOccupied);
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

    /// Get the next player
    pub const fn next_player(&self) -> Player {
        self.next_player
    }

    /// Get game status
    pub const fn status(&self) -> &Status {
        &self.status
    }

    fn update_status(&mut self, last_move: LastMove) {
        // to determine if the game is ended by the last move, 3 positions centered at the last move are checked on each direction

        // horizontal
        if self.get(last_move.row, 0) == self.get(last_move.row, 1)
            && self.get(last_move.row, 1) == self.get(last_move.row, 2)
        {
            self.status = Status::Win(last_move.player);
            return;
        }

        // vertical
        if self.get(0, last_move.col) == self.get(1, last_move.col)
            && self.get(1, last_move.col) == self.get(2, last_move.col)
        {
            self.status = Status::Win(last_move.player);
            return;
        }

        // check diagonal only if the last move is on the diagonal
        if !((last_move.row == 1) ^ (last_move.col == 1)) {
            // top-left to bottom-right diagonal
            if self.get(0, 0) == self.get(1, 1) && self.get(1, 1) == self.get(2, 2) {
                self.status = Status::Win(last_move.player);
                return;
            }

            // top-right to bottom-left diagonal
            if self.get(0, 2) == self.get(1, 1) && self.get(1, 1) == self.get(2, 0) {
                self.status = Status::Win(last_move.player);
                return;
            }
        }

        // check draw
        if self.move_count == BOARD_HEIGHT * BOARD_WIDTH {
            self.status = Status::Draw;
        }
    }
}

impl Player {
    /// Get the opposite player
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
        let mut game = TicTacToe::new().unwrap();

        game.put(1, 1).unwrap();

        assert_eq!(game.next_player(), Player::Player1);

        game.put(1, 0).unwrap();

        assert_eq!(game.next_player(), Player::Player0);
        assert_eq!(game.put(1, 1), Err(TicTacToeError::PositionOccupied));

        game.put(2, 2).unwrap();
        game.put(2, 0).unwrap();
        game.put(0, 0).unwrap();

        assert_eq!(game.status(), &Status::Win(Player::Player0));
        assert_eq!(game.put(0, 2), Err(TicTacToeError::GameEnded));
    }
}
