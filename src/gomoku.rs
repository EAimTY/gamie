//! Gomoku.

use crate::std_lib::Infallible;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use snafu::Snafu;

/// The Gomoku game.
///
/// Passing an invalid position to a method could cause a panic. Remember to check the target position validity when dealing with user input.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Gomoku {
    board: [[Option<Player>; 15]; 15],
    next: Player,
    state: GameState,
}

/// The Gomoku game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Player {
    Player0,
    Player1,
}

impl Player {
    /// Get the opposite player.
    pub fn other(self) -> Self {
        match self {
            Player::Player0 => Player::Player1,
            Player::Player1 => Player::Player0,
        }
    }
}

/// The Gomoku game state.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GameState {
    Win(Player),
    Tie,
    InProgress,
}

impl Gomoku {
    /// Create a new Gomoku game.
    pub fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: [[None; 15]; 15],
            next: Player::Player0,
            state: GameState::InProgress,
        })
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Option<Player> {
        &self.board[row][col]
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.state != GameState::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not ended.
    pub fn winner(&self) -> Option<Player> {
        if let GameState::Win(player) = self.state {
            Some(player)
        } else {
            None
        }
    }

    /// Get the state of the game.
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> Player {
        self.next
    }

    /// Place a piece on the board.
    pub fn place(&mut self, player: Player, row: usize, col: usize) -> Result<(), GomokuError> {
        if self.is_ended() {
            return Err(GomokuError::GameEnded);
        }

        if player != self.next {
            return Err(GomokuError::WrongPlayer);
        }

        if self.board[row][col].is_some() {
            return Err(GomokuError::PositionOccupied);
        }

        self.board[row][col] = Some(player);
        self.next = self.next.other();

        self.check_state();

        Ok(())
    }

    fn check_state(&mut self) {
        todo!();
    }
}

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, PartialEq, Snafu)]
pub enum GomokuError {
    #[snafu(display("Wrong player"))]
    WrongPlayer,
    #[snafu(display("Position already been occupied"))]
    PositionOccupied,
    #[snafu(display("The game was already end"))]
    GameEnded,
}
