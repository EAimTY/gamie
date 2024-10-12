//! Connect Four
//!
//! Check struct [`ConnectFour`] for more information

use core::{
    convert::Infallible,
    fmt::{Debug, Formatter, Result as FmtResult},
};
use snafu::Snafu;

const BOARD_WIDTH: usize = 7;
const BOARD_HEIGHT: usize = 6;

/// Connect Four
///
/// # Examples
///
/// ```rust
/// # use gamie::connect_four::ConnectFour;
/// let mut game = ConnectFour::new().unwrap();
/// game.put(3).unwrap();
/// game.put(2).unwrap();
/// // ...
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConnectFour {
    columns: [Column; BOARD_WIDTH],
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

/// Errors that can occur when putting a piece onto the board
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum ConnectFourError {
    #[snafu(display("column filled"))]
    ColumnFilled,
    #[snafu(display("game ended"))]
    GameEnded,
}

/// Column in the board
///
/// Since pieces are placed from the bottom, the column is represented as a grow-only stack
/// `Option<Player>` is not needed since we are tracking the number of filled cells
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

impl ConnectFour {
    /// Create a new Connect Four game
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

    /// Get a piece at a position
    ///
    /// Panic if the target position is out of bounds
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        let column = &self.columns[col];

        if row >= BOARD_HEIGHT - column.filled {
            Some(column.cells[row])
        } else {
            None
        }
    }

    /// Put a piece
    ///
    /// Panic if the target position is out of bounds
    pub fn put(&mut self, col: usize) -> Result<(), ConnectFourError> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(ConnectFourError::GameEnded);
        }

        if self.columns[col].filled == BOARD_HEIGHT {
            return Err(ConnectFourError::ColumnFilled);
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

    /// Get the next player
    pub const fn next_player(&self) -> Player {
        self.next_player
    }

    /// Get game status
    pub const fn status(&self) -> &Status {
        &self.status
    }

    fn update_status(&mut self, last_move: LastMove) {
        // to determine if the game is ended by the last move, 7 positions centered at the last move are checked on each direction

        let checking_row_range =
            last_move.row.saturating_sub(3)..=(last_move.row + 3).min(BOARD_HEIGHT - 1);
        let checking_col_range =
            last_move.col.saturating_sub(3)..=(last_move.col + 3).min(BOARD_WIDTH - 1);
        let mut continuous_player_pieces = 0;

        // horizontal
        for col in checking_col_range.clone() {
            if self.get(last_move.row, col) == Some(last_move.player) {
                continuous_player_pieces += 1;
                if continuous_player_pieces == 4 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // vertical
        continuous_player_pieces = 0;

        for row in checking_row_range.clone() {
            if self.get(row, last_move.col) == Some(last_move.player) {
                continuous_player_pieces += 1;
                if continuous_player_pieces == 4 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // top-left to bottom-right diagonal
        continuous_player_pieces = 0;

        for (row, col) in checking_row_range.clone().zip(checking_col_range.clone()) {
            if self.get(row, col) == Some(last_move.player) {
                continuous_player_pieces += 1;
                if continuous_player_pieces == 4 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // top-right to bottom-left diagonal
        continuous_player_pieces = 0;

        for (row, col) in checking_row_range.zip(checking_col_range.rev()) {
            if self.get(row, col) == Some(last_move.player) {
                continuous_player_pieces += 1;
                if continuous_player_pieces == 4 {
                    self.status = Status::Win(last_move.player);
                    return;
                }
            }
        }

        // check draw
        if self.move_count == BOARD_HEIGHT * BOARD_WIDTH {
            self.status = Status::Draw;
        }
    }
}

impl Player {
    /// Get the other player
    pub const fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
        }
    }
}

impl Debug for ConnectFour {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut board = [[None; BOARD_HEIGHT]; BOARD_WIDTH];

        for col in 0..BOARD_WIDTH {
            let column = &self.columns[col];

            for row in 0..column.filled {
                board[col][row] = Some(column.cells[row]);
            }
        }

        f.debug_struct("ConnectFour")
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

    #[test]
    fn test() {
        let mut game = ConnectFour::new().unwrap();

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
}
