//! Gomoku
//!
//! Check struct [`Gomoku`] for more information

use core::convert::Infallible;
use snafu::Snafu;

const BOARD_WIDTH: usize = 15;
const BOARD_HEIGHT: usize = 15;

/// Gomoku
///
/// # Examples
///
/// ```rust
/// # use gamie::gomoku::Gomoku;
/// let mut game = Gomoku::new().unwrap();
/// game.put(7, 8).unwrap();
/// game.put(8, 7).unwrap();
/// // ...
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Gomoku {
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

/// Errors that can occur when placing a piece on the board
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum GomokuError {
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

impl Gomoku {
    /// Create a new Gomoku game.
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
    pub fn get(&self, row: usize, col: usize) -> Option<Player> {
        self.board[row][col]
    }

    /// Put a piece
    ///
    /// Panic if the target position is out of bounds
    pub fn put(&mut self, row: usize, col: usize) -> Result<(), GomokuError> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(GomokuError::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(GomokuError::PositionOccupied);
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
    pub fn next_player(&self) -> Player {
        self.next_player
    }

    /// Get game status
    pub fn status(&self) -> &Status {
        &self.status
    }

    fn update_status(&mut self, last_move: LastMove) {
        // to determine if the game is ended by the last move, 9 positions centered at the last move are checked on each direction

        let checking_row_range =
            last_move.row.saturating_sub(4)..=(last_move.row + 4).min(BOARD_HEIGHT - 1);
        let checking_col_range =
            last_move.col.saturating_sub(4)..=(last_move.col + 4).min(BOARD_WIDTH - 1);
        let mut continuous_player_pieces = 0;

        // horizontal
        for col in checking_col_range.clone() {
            if self.get(last_move.row, col) == Some(last_move.player) {
                continuous_player_pieces += 1;
                if continuous_player_pieces == 5 {
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
                if continuous_player_pieces == 5 {
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
                if continuous_player_pieces == 5 {
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
                if continuous_player_pieces == 5 {
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
    pub fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
        }
    }
}
