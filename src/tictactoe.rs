//! The Tic-Tac-Toe game.
//!
//! Check out struct [`TicTacToe`](https://docs.rs/gamie/*/gamie/tictactoe/struct.TicTacToe.html) for more information.
//!
//! # Examples
//!
//! ```rust
//! use gamie::tictactoe::*;
//!
//! # fn tictactoe() {
//! let mut game = TicTacToe::new().unwrap();
//!
//! game.place(TicTacToePlayer::X, 1, 1).unwrap();
//! game.place(TicTacToePlayer::O, 0, 0).unwrap();
//!
//! // ...
//!
//! println!(game.state());
//! # }
//! ```

/// The Tic-Tac-Toe game.
/// If you pass an invalid position to a method, the game will panic. Remember to check the target position validity when dealing with user input.
#[derive(Clone, Debug)]
pub struct TicTacToe {
    pub board: [[Option<TicTacToePlayer>; 3]; 3],
    pub next: TicTacToePlayer,
    pub state: TicTacToeState,
}

/// The Tic-Tac-Toe game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicTacToePlayer {
    X,
    O,
}

impl TicTacToePlayer {
    /// Get the opposite player.
    pub fn other(self) -> Self {
        match self {
            TicTacToePlayer::X => TicTacToePlayer::O,
            TicTacToePlayer::O => TicTacToePlayer::X,
        }
    }
}

/// The Tic-Tac-Toe game state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TicTacToeState {
    Win(TicTacToePlayer),
    Tie,
    InProgress,
}

use std::convert::Infallible;

impl TicTacToe {
    /// Create a new Tic-Tac-Toe game.
    pub fn new() -> Result<Self, Infallible> {
        Ok(Self {
            board: [[None; 3]; 3],
            next: TicTacToePlayer::X,
            state: TicTacToeState::InProgress,
        })
    }

    /// Get a cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get(&self, row: usize, col: usize) -> &Option<TicTacToePlayer> {
        &self.board[row][col]
    }

    /// Get a mutable cell reference from the game board.
    /// Panic if the target position is out of bounds.
    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut Option<TicTacToePlayer> {
        &mut self.board[row][col]
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.state != TicTacToeState::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not ended.
    pub fn winner(&self) -> Option<TicTacToePlayer> {
        if let TicTacToeState::Win(player) = self.state {
            Some(player)
        } else {
            None
        }
    }

    /// Get the state of the game.
    pub fn state(&self) -> &TicTacToeState {
        &self.state
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> TicTacToePlayer {
        self.next
    }

    /// Place a piece on the board.
    pub fn place(
        &mut self,
        player: TicTacToePlayer,
        row: usize,
        col: usize,
    ) -> Result<(), TicTacToeError> {
        if self.is_ended() {
            return Err(TicTacToeError::GameEnded);
        }

        if player != self.next {
            return Err(TicTacToeError::WrongPlayer);
        }

        if self.board[row][col].is_some() {
            return Err(TicTacToeError::PositionOccupied);
        }

        self.board[row][col] = Some(player);
        self.next = self.next.other();

        self.check_state();

        Ok(())
    }

    fn check_state(&mut self) {
        for row in 0..3 {
            if self.board[row][0].is_some()
                && self.board[row][0] == self.board[row][1]
                && self.board[row][1] == self.board[row][2]
            {
                self.state = TicTacToeState::Win(self.board[row][0].unwrap());
                return;
            }
        }

        for col in 0..3 {
            if self.board[0][col].is_some()
                && self.board[0][col] == self.board[1][col]
                && self.board[1][col] == self.board[2][col]
            {
                self.state = TicTacToeState::Win(self.board[0][col].unwrap());
                return;
            }
        }

        if self.board[0][0].is_some()
            && self.board[0][0] == self.board[1][1]
            && self.board[1][1] == self.board[2][2]
        {
            self.state = TicTacToeState::Win(self.board[0][0].unwrap());
            return;
        }

        if self.board[0][0].is_some()
            && self.board[0][2] == self.board[1][1]
            && self.board[1][1] == self.board[2][0]
        {
            self.state = TicTacToeState::Win(self.board[0][2].unwrap());
            return;
        }

        self.state = if self.board.iter().flatten().all(|p| p.is_some()) {
            TicTacToeState::Tie
        } else {
            TicTacToeState::InProgress
        };
    }
}

use thiserror::Error;

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum TicTacToeError {
    #[error("Wrong player")]
    WrongPlayer,
    #[error("Position already been occupied")]
    PositionOccupied,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::tictactoe::*;

    #[test]
    fn test() {
        let mut game = TicTacToe::new().unwrap();

        assert_eq!(game.get_next_player(), TicTacToePlayer::X,);

        assert_eq!(game.place(TicTacToePlayer::X, 1, 1), Ok(()));

        assert_eq!(game.get_next_player(), TicTacToePlayer::O,);

        assert_eq!(
            game.place(TicTacToePlayer::X, 0, 0),
            Err(TicTacToeError::WrongPlayer)
        );

        assert_eq!(game.place(TicTacToePlayer::O, 1, 0), Ok(()));

        assert_eq!(game.get_next_player(), TicTacToePlayer::X,);

        assert!(!game.is_ended());

        assert_eq!(
            game.place(TicTacToePlayer::X, 1, 1),
            Err(TicTacToeError::PositionOccupied)
        );

        assert_eq!(game.place(TicTacToePlayer::X, 2, 2), Ok(()));

        assert_eq!(game.state(), &TicTacToeState::InProgress);

        assert_eq!(game.place(TicTacToePlayer::O, 2, 0), Ok(()));

        assert_eq!(game.place(TicTacToePlayer::X, 0, 0), Ok(()));

        assert!(game.is_ended());

        assert_eq!(game.winner(), Some(TicTacToePlayer::X));

        assert_eq!(
            game.place(TicTacToePlayer::X, 0, 2),
            Err(TicTacToeError::GameEnded)
        );

        assert_eq!(game.winner(), Some(TicTacToePlayer::X));
    }
}
