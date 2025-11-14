//! Reversi (Othello) game implementation
//!
//! Reversi is a strategy board game for two players, played on an 8×8 board.
//! Players take turns placing pieces on the board, flipping opponent pieces
//! that are surrounded by the current player's pieces in any direction
//! (horizontal, vertical, or diagonal).
//!
//! See [`Game`] for the main game interface.

use core::{cmp::Ordering, convert::Infallible};
use thiserror::Error;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

/// A Reversi (Othello) game instance
///
/// The game is played on an 8×8 board where two players alternate placing their pieces.
/// Player0 always goes first. The game ends when neither player can make a valid move,
/// and the player with the most pieces wins.
///
/// # Examples
///
/// ```rust
/// # use gamie::reversi::{Player, Game};
/// let mut game = Game::new().unwrap();
///
/// // Player0's turn
/// game.place(2, 4).unwrap();
///
/// // Player1's turn
/// game.place(2, 3).unwrap();
///
/// // Continue playing...
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    board: [[Option<Player>; BOARD_HEIGHT]; BOARD_WIDTH],
    next_player: Player,
    status: Status,
}

/// Represents a player in the game
///
/// There are two players in Reversi. Player0 always makes the first move,
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
/// - [`Win`](Status::Win): A player has won by having the most pieces
/// - [`Draw`](Status::Draw): Both players have the same number of pieces
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
    /// The move would not flip any opponent pieces
    #[error("invalid position")]
    InvalidPosition,
    /// The game has already ended (either in a win or draw)
    #[error("game ended")]
    GameEnded,
}

impl Game {
    /// Creates a new Reversi game with the standard starting position
    ///
    /// The board is initialized with 4 pieces in the center:
    /// - Player0 at positions (3,3) and (4,4)
    /// - Player1 at positions (3,4) and (4,3)
    ///
    /// Player0 will make the first move.
    pub const fn new() -> Result<Self, Infallible> {
        let mut board = [[None; BOARD_HEIGHT]; BOARD_WIDTH];

        board[3][3] = Some(Player::Player0);
        board[4][4] = Some(Player::Player0);
        board[3][4] = Some(Player::Player1);
        board[4][3] = Some(Player::Player1);

        Ok(Self {
            board,
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
    /// - `row`: The row index (0-7)
    /// - `col`: The column index (0-7)
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than or equal to 8)
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        self.board[row][col]
    }

    /// Places a piece for the current player at the specified position
    ///
    /// The current player's piece (returned by [`next_player()`](Self::next_player))
    /// is placed at the given position and flips all opponent pieces that are captured
    /// in any direction (horizontal, vertical, or diagonal). After a successful placement,
    /// the turn automatically switches to the other player (if they have valid moves).
    ///
    /// # Parameters
    ///
    /// - `row`: The row index (0-7)
    /// - `col`: The column index (0-7)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The position is already occupied ([`Error::PositionOccupied`])
    /// - The move would not flip any opponent pieces ([`Error::InvalidPosition`])
    /// - The game has already ended ([`Error::GameEnded`])
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than or equal to 8)
    pub fn place(&mut self, row: usize, col: usize) -> Result<(), Error> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(Error::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(Error::PositionOccupied);
        }

        let flipping_left_range = (0..col).rev();
        let flipping_right_range = col + 1..BOARD_WIDTH;
        let flipping_up_range = (0..row).rev();
        let flipping_down_range = row + 1..BOARD_HEIGHT;

        let mut any_flipped = false;

        // flip left
        any_flipped |= self.try_flip_line(flipping_left_range.clone().map(|col| (row, col)));

        // flip right
        any_flipped |= self.try_flip_line(flipping_right_range.clone().map(|col| (row, col)));

        // flip up
        any_flipped |= self.try_flip_line(flipping_up_range.clone().map(|row| (row, col)));

        // flip down
        any_flipped |= self.try_flip_line(flipping_down_range.clone().map(|row| (row, col)));

        // flip upper left
        any_flipped |=
            self.try_flip_line(flipping_up_range.clone().zip(flipping_left_range.clone()));

        // flip upper right
        any_flipped |=
            self.try_flip_line(flipping_up_range.clone().zip(flipping_right_range.clone()));

        // flip lower left
        any_flipped |=
            self.try_flip_line(flipping_down_range.clone().zip(flipping_left_range.clone()));

        // flip lower right
        any_flipped |= self.try_flip_line(flipping_down_range.zip(flipping_right_range));

        if !any_flipped {
            return Err(Error::InvalidPosition);
        }

        // place the piece
        self.board[row][col] = Some(self.next_player);

        self.next_player = self.next_player.other();
        if self.can_current_player_move() {
            return Ok(());
        }

        self.next_player = self.next_player.other();
        if self.can_current_player_move() {
            return Ok(());
        }

        // Neither player can move; the game ends
        let mut player0_count = 0u8;
        let mut player1_count = 0u8;

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                match self.get(row, col) {
                    Some(Player::Player0) => player0_count += 1,
                    Some(Player::Player1) => player1_count += 1,
                    None => {}
                }
            }
        }

        match player0_count.cmp(&player1_count) {
            Ordering::Greater => self.status = Status::Win(Player::Player0),
            Ordering::Less => self.status = Status::Win(Player::Player1),
            Ordering::Equal => self.status = Status::Draw,
        }

        Ok(())
    }

    /// Checks if the current player can place a piece at the specified position
    ///
    /// This method validates whether placing a piece at the given position would
    /// be a legal move for the current player (without actually making the move).
    ///
    /// # Parameters
    ///
    /// - `row`: The row index (0-7)
    /// - `col`: The column index (0-7)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The position is already occupied ([`Error::PositionOccupied`])
    /// - The move would not flip any opponent pieces ([`Error::InvalidPosition`])
    /// - The game has already ended ([`Error::GameEnded`])
    ///
    /// # Panics
    ///
    /// Panics if `row` or `col` is out of bounds (greater than or equal to 8)
    pub fn can_place_at(&self, row: usize, col: usize) -> Result<(), Error> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(Error::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(Error::PositionOccupied);
        }

        // check each direction for a valid move

        let checking_left_range = (0..col).rev();
        let checking_right_range = col + 1..BOARD_WIDTH;
        let checking_up_range = (0..row).rev();
        let checking_down_range = row + 1..BOARD_HEIGHT;

        // check left
        if self.can_flip_line(checking_left_range.clone().map(|col| (row, col))) {
            return Ok(());
        }

        // check right
        if self.can_flip_line(checking_right_range.clone().map(|col| (row, col))) {
            return Ok(());
        }

        // check up
        if self.can_flip_line(checking_up_range.clone().map(|row| (row, col))) {
            return Ok(());
        }

        // check down
        if self.can_flip_line(checking_down_range.clone().map(|row| (row, col))) {
            return Ok(());
        }

        // check upper left
        if self.can_flip_line(checking_up_range.clone().zip(checking_left_range.clone())) {
            return Ok(());
        }

        // check upper right
        if self.can_flip_line(checking_up_range.clone().zip(checking_right_range.clone())) {
            return Ok(());
        }

        // check lower left
        if self.can_flip_line(checking_down_range.clone().zip(checking_left_range.clone())) {
            return Ok(());
        }

        // check lower right
        if self.can_flip_line(checking_down_range.zip(checking_right_range)) {
            return Ok(());
        }

        Err(Error::InvalidPosition)
    }

    /// Gets the next player whose turn it is to move
    ///
    /// Returns the player who should make the next move. This changes
    /// after each successful call to [`place()`](Self::place).
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

    fn can_current_player_move(&self) -> bool {
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                match self.can_place_at(row, col) {
                    Err(Error::PositionOccupied | Error::InvalidPosition) => continue,
                    Ok(()) => return true,
                    Err(Error::GameEnded) => unreachable!(),
                }
            }
        }

        false
    }

    fn try_flip_line(&mut self, line: impl Iterator<Item = (usize, usize)> + Clone) -> bool {
        let mut skipped = 0;

        let Some((row, col)) = line
            .clone()
            .skip_while(|(row, col)| {
                let is_other_player = self.get(*row, *col) == Some(self.next_player().other());
                skipped += is_other_player as usize;
                is_other_player
            })
            .next()
        else {
            return false;
        };

        if skipped == 0 || self.get(row, col) != Some(self.next_player()) {
            return false;
        }

        for (row, col) in line.take(skipped) {
            self.board[row][col] = Some(self.next_player());
        }

        true
    }

    fn can_flip_line(&self, line: impl Iterator<Item = (usize, usize)>) -> bool {
        let mut skipped = false;

        let Some((row, col)) = line
            .skip_while(|(row, col)| {
                let is_other_player = self.get(*row, *col) == Some(self.next_player().other());
                skipped |= is_other_player;
                is_other_player
            })
            .next()
        else {
            return false;
        };

        skipped && self.get(row, col) == Some(self.next_player())
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
    use crate::reversi::*;

    #[test]
    fn test() {
        let mut game = Game::new().unwrap();

        game.can_place_at(2, 4).unwrap();

        game.place(2, 4).unwrap();
        game.place(2, 3).unwrap();

        assert!(matches!(game.place(2, 3), Err(Error::PositionOccupied)));
        assert!(matches!(game.place(2, 6), Err(Error::InvalidPosition)));
    }
}
