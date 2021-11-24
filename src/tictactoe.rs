/// The Tic-Tac-Toe game.
///
/// # Examples
///
/// ```rust
/// use boards::tictactoe::{TicTacToe, TicTacToePlayer, TicTacToeStatus, TicTacToeError};
///
/// fn main() {
///     let mut game = TicTacToe::new();
///
///     game.place(TicTacToePlayer::X, 1, 1).unwrap();
///     game.place(TicTacToePlayer::O, 0, 0).unwrap();
///
///     // ...
///
///     dbg!(game.status());
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TicTacToe {
    pub board: [[Option<TicTacToePlayer>; 3]; 3],
    pub next: TicTacToePlayer,
    pub status: TicTacToeStatus,
}

/// The Tic-Tac-Toe game players.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicTacToePlayer {
    X,
    O,
}

impl TicTacToePlayer {
    pub fn other(self) -> Self {
        match self {
            TicTacToePlayer::X => TicTacToePlayer::O,
            TicTacToePlayer::O => TicTacToePlayer::X,
        }
    }
}

/// The Tic-Tac-Toe game status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TicTacToeStatus {
    Win(TicTacToePlayer),
    Tie,
    InProgress,
}

impl TicTacToe {
    /// Create a new Tic-Tac-Toe game.
    pub fn new() -> Self {
        TicTacToe {
            board: [[None; 3]; 3],
            next: TicTacToePlayer::X,
            status: TicTacToeStatus::InProgress,
        }
    }

    /// Check if the game is ended.
    pub fn is_ended(&self) -> bool {
        self.status != TicTacToeStatus::InProgress
    }

    /// Get the winner of the game. Return `None` if the game is tied or not ended.
    pub fn winner(&self) -> Option<TicTacToePlayer> {
        if let TicTacToeStatus::Win(player) = self.status {
            Some(player)
        } else {
            None
        }
    }

    /// Get the status of the game.
    pub fn status(&self) -> TicTacToeStatus {
        self.status
    }

    /// Get the next player.
    pub fn get_next_player(&self) -> TicTacToePlayer {
        self.next
    }

    /// Get the board.
    pub fn board(&self) -> &[[Option<TicTacToePlayer>; 3]; 3] {
        &self.board
    }

    /// Place a piece on the board.
    pub fn place(
        &mut self,
        player: TicTacToePlayer,
        row: usize,
        col: usize,
    ) -> Result<TicTacToeStatus, TicTacToeError> {
        if self.is_ended() {
            return Err(TicTacToeError::GameEnded);
        }

        if player != self.next {
            return Err(TicTacToeError::WrongPlayer);
        }

        if row > 2 || col > 2 {
            return Err(TicTacToeError::OutOfBounds);
        }

        if self.board[row][col].is_some() {
            return Err(TicTacToeError::PositionOccupied);
        }

        self.board[row][col] = Some(player);
        self.next = self.next.other();

        self.status = self.check_win();

        Ok(self.status)
    }

    fn check_win(&self) -> TicTacToeStatus {
        for row in 0..3 {
            if self.board[row][0].is_some()
                && self.board[row][0] == self.board[row][1]
                && self.board[row][1] == self.board[row][2]
            {
                return TicTacToeStatus::Win(self.board[row][0].unwrap());
            }
        }

        for col in 0..3 {
            if self.board[0][col].is_some()
                && self.board[0][col] == self.board[1][col]
                && self.board[1][col] == self.board[2][col]
            {
                return TicTacToeStatus::Win(self.board[0][col].unwrap());
            }
        }

        if self.board[0][0].is_some()
            && self.board[0][0] == self.board[1][1]
            && self.board[1][1] == self.board[2][2]
        {
            return TicTacToeStatus::Win(self.board[0][0].unwrap());
        }

        if self.board[0][0].is_some()
            && self.board[0][2] == self.board[1][1]
            && self.board[1][1] == self.board[2][0]
        {
            return TicTacToeStatus::Win(self.board[0][2].unwrap());
        }

        if self.board.iter().flatten().all(|p| p.is_some()) {
            TicTacToeStatus::Tie
        } else {
            TicTacToeStatus::InProgress
        }
    }
}

use thiserror::Error;

/// Errors that can occur when placing a piece on the board.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum TicTacToeError {
    #[error("Wrong player")]
    WrongPlayer,
    #[error("Position out of bounds")]
    OutOfBounds,
    #[error("Position already been occupied")]
    PositionOccupied,
    #[error("The game was already ended")]
    GameEnded,
}

#[cfg(test)]
mod tests {
    use crate::{TicTacToe, TicTacToeError, TicTacToePlayer, TicTacToeStatus};

    #[test]
    fn win() {
        let mut game = TicTacToe::new();

        assert_eq!(game.get_next_player(), TicTacToePlayer::X,);

        assert_eq!(
            game.place(TicTacToePlayer::X, 1, 1),
            Ok(TicTacToeStatus::InProgress)
        );

        assert_eq!(game.get_next_player(), TicTacToePlayer::O,);

        assert_eq!(
            game.place(TicTacToePlayer::X, 0, 0),
            Err(TicTacToeError::WrongPlayer)
        );

        assert_eq!(
            game.place(TicTacToePlayer::O, 1, 0),
            Ok(TicTacToeStatus::InProgress)
        );

        assert_eq!(game.get_next_player(), TicTacToePlayer::X,);

        assert!(!game.is_ended());

        assert_eq!(
            game.place(TicTacToePlayer::X, 1, 1),
            Err(TicTacToeError::PositionOccupied)
        );

        assert_eq!(
            game.place(TicTacToePlayer::X, 2, 2),
            Ok(TicTacToeStatus::InProgress)
        );

        assert_eq!(
            game.place(TicTacToePlayer::O, 3, 0),
            Err(TicTacToeError::OutOfBounds)
        );

        assert_eq!(game.status(), TicTacToeStatus::InProgress);

        assert_eq!(
            game.place(TicTacToePlayer::O, 2, 0),
            Ok(TicTacToeStatus::InProgress)
        );

        assert_eq!(
            game.place(TicTacToePlayer::X, 0, 0),
            Ok(TicTacToeStatus::Win(TicTacToePlayer::X))
        );

        assert!(game.is_ended());

        assert_eq!(
            game.place(TicTacToePlayer::X, 0, 2),
            Err(TicTacToeError::GameEnded)
        );

        assert_eq!(game.winner(), Some(TicTacToePlayer::X));
    }

    #[test]
    fn tie() {
        let mut game = TicTacToe::new();

        game.place(TicTacToePlayer::X, 1, 1).unwrap();
        game.place(TicTacToePlayer::O, 0, 0).unwrap();
        game.place(TicTacToePlayer::X, 0, 2).unwrap();
        game.place(TicTacToePlayer::O, 2, 0).unwrap();
        game.place(TicTacToePlayer::X, 1, 0).unwrap();
        game.place(TicTacToePlayer::O, 1, 2).unwrap();
        game.place(TicTacToePlayer::X, 2, 1).unwrap();
        game.place(TicTacToePlayer::O, 0, 1).unwrap();
        game.place(TicTacToePlayer::X, 2, 2).unwrap();

        assert!(game.is_ended());
        assert_eq!(game.status(), TicTacToeStatus::Tie);
    }
}
