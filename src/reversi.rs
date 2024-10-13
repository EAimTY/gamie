//! Reversi
//!
//! Check struct [`Reversi`] for more information

use core::{cmp::Ordering, convert::Infallible};
use snafu::Snafu;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

/// Reversi
///
/// # Examples
///
/// ```rust
/// # use gamie::reversi::{Player, Reversi};
/// let mut game = Reversi::new().unwrap();
///
/// game.put(2, 4).unwrap();
///
///
/// // The next player may not be able to place a piece onto any position, so manually check `next_player()` to determine the next player
/// assert_eq!(game.next_player(), Player::Player1);
///
/// game.put(2, 3).unwrap();
/// // ...
/// ```
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reversi {
    board: [[Option<Player>; BOARD_HEIGHT]; BOARD_WIDTH],
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
pub enum ReversiError {
    #[snafu(display("position occupied"))]
    PositionOccupied,
    #[snafu(display("invalid position"))]
    InvalidPosition,
    #[snafu(display("game ended"))]
    GameEnded,
}

impl Reversi {
    /// Create a new Reversi game
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

    /// Get a piece at a position
    ///
    /// Panic if the target position is out of bounds
    pub const fn get(&self, row: usize, col: usize) -> Option<Player> {
        self.board[row][col]
    }

    /// Put a piece
    ///
    /// Panic if the target position is out of bounds
    pub fn put(&mut self, row: usize, col: usize) -> Result<(), ReversiError> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(ReversiError::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(ReversiError::PositionOccupied);
        }

        let flipping_left_range = (0..col).rev();
        let flipping_right_range = col + 1..BOARD_WIDTH;
        let flipping_up_range = (0..row).rev();
        let flipping_down_range = row + 1..BOARD_HEIGHT;

        let mut is_flipped = false;

        // flip left
        is_flipped |= self.flip_in_line(flipping_left_range.clone().map(|col| (row, col)));

        // flip right
        is_flipped |= self.flip_in_line(flipping_right_range.clone().map(|col| (row, col)));

        // flip up
        is_flipped |= self.flip_in_line(flipping_up_range.clone().map(|row| (row, col)));

        // flip down
        is_flipped |= self.flip_in_line(flipping_down_range.clone().map(|row| (row, col)));

        // flip upper left
        is_flipped |= self.flip_in_line(flipping_up_range.clone().zip(flipping_left_range.clone()));

        // flip upper right
        is_flipped |=
            self.flip_in_line(flipping_up_range.clone().zip(flipping_right_range.clone()));

        // flip lower left
        is_flipped |=
            self.flip_in_line(flipping_down_range.clone().zip(flipping_left_range.clone()));

        // flip lower right
        is_flipped |= self.flip_in_line(flipping_down_range.zip(flipping_right_range));

        if !is_flipped {
            return Err(ReversiError::InvalidPosition);
        }

        // place the piece
        self.board[row][col] = Some(self.next_player);

        self.next_player = self.next_player.other();
        if self.is_current_player_movable() {
            return Ok(());
        }

        self.next_player = self.next_player.other();
        if self.is_current_player_movable() {
            return Ok(());
        }

        // both players cannot move, game ends
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

    /// Check if target position is valid for placing a piece
    ///
    /// Panic if the target position is out of bounds
    pub fn is_position_valid_for_put(&self, row: usize, col: usize) -> Result<(), ReversiError> {
        if matches!(self.status, Status::Win(_) | Status::Draw) {
            return Err(ReversiError::GameEnded);
        }

        if self.board[row][col].is_some() {
            return Err(ReversiError::PositionOccupied);
        }

        // check each direction for a valid move

        let checking_left_range = (0..col).rev();
        let checking_right_range = col + 1..BOARD_WIDTH;
        let checking_up_range = (0..row).rev();
        let checking_down_range = row + 1..BOARD_HEIGHT;

        // check left
        if self.is_clipping_in_line(checking_left_range.clone().map(|col| (row, col))) {
            return Ok(());
        }

        // check right
        if self.is_clipping_in_line(checking_right_range.clone().map(|col| (row, col))) {
            return Ok(());
        }

        // check up
        if self.is_clipping_in_line(checking_up_range.clone().map(|row| (row, col))) {
            return Ok(());
        }

        // check down
        if self.is_clipping_in_line(checking_down_range.clone().map(|row| (row, col))) {
            return Ok(());
        }

        // check upper left
        if self.is_clipping_in_line(checking_up_range.clone().zip(checking_left_range.clone())) {
            return Ok(());
        }

        // check upper right
        if self.is_clipping_in_line(checking_up_range.clone().zip(checking_right_range.clone())) {
            return Ok(());
        }

        // check lower left
        if self.is_clipping_in_line(checking_down_range.clone().zip(checking_left_range.clone())) {
            return Ok(());
        }

        // check lower right
        if self.is_clipping_in_line(checking_down_range.zip(checking_right_range)) {
            return Ok(());
        }

        Err(ReversiError::InvalidPosition)
    }

    /// Get the next player
    pub const fn next_player(&self) -> Player {
        self.next_player
    }

    /// Get game status
    pub const fn status(&self) -> &Status {
        &self.status
    }

    fn is_current_player_movable(&self) -> bool {
        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                match self.is_position_valid_for_put(row, col) {
                    Err(ReversiError::PositionOccupied | ReversiError::InvalidPosition) => continue,
                    Ok(()) => return true,
                    Err(ReversiError::GameEnded) => unreachable!(),
                }
            }
        }

        false
    }

    fn flip_in_line(&mut self, line: impl Iterator<Item = (usize, usize)> + Clone) -> bool {
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

    fn is_clipping_in_line(&self, line: impl Iterator<Item = (usize, usize)>) -> bool {
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
    /// Get the other player
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
        let mut game = Reversi::new().unwrap();

        game.is_position_valid_for_put(2, 4).unwrap();

        game.put(2, 4).unwrap();
        game.put(2, 3).unwrap();

        assert_eq!(game.put(2, 3), Err(ReversiError::PositionOccupied));
        assert_eq!(game.put(2, 6), Err(ReversiError::InvalidPosition));
    }
}
