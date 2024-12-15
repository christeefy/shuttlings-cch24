use std::fmt;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use ndarray::{s, Array2};
use serde::Deserialize;

use crate::AppState;

const MILK_ICON: &str = "🥛";
const COOKIE_ICON: &str = "🍪";
const EMPTY_ICON: &str = "⬛";
const WALL_ICON: &str = "⬜";

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Cookie,
    Milk,
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let emoji = match self {
            Self::Cookie => COOKIE_ICON,
            Self::Milk => MILK_ICON,
        };
        write!(f, "{emoji}")?;
        Ok(())
    }
}

enum GameState {
    Won(Team),
    Stalemate,
    NotYetWon,
}

impl Default for GameState {
    fn default() -> Self {
        Self::NotYetWon
    }
}

/// Create an `N`x`N` board, with an extra layer of walls (left, right and bottom).
/// Implemented using const generics.
pub struct Board<const N: usize = 4> {
    cells: Array2<Option<Team>>,
    game_state: GameState,
}

// NOTE: This doesn't impl `Error`
pub enum BoardError {
    ColumnFull(usize),
    OutOfBound(usize),
    GameOver,
}

impl<const N: usize> Board<N> {
    pub fn new() -> Self {
        Self {
            cells: Array2::default((N, N)),
            game_state: GameState::default(),
        }
    }

    pub fn reset(&mut self) {
        self.cells = Array2::default((N, N));
        self.game_state = GameState::default();
    }

    #[inline]
    pub fn size(&self) -> usize {
        N
    }

    // pub fn set(&mut self, value: Team, (x, y): (usize, usize)) {
    //     if let WinState::Won(ref team) = self.win_state {
    //         panic!("Cannot set board. Game is already complete with {team} as the winner")
    //     }
    //     match self.cells.get((x, y)) {
    //         Some(Some(_)) => panic!("Tile ({x}, {y}) is already set"),
    //         Some(None) => self.cells[[x, y]] = Some(value.clone()),
    //         None => panic!("Out of bounds ({x}, {y})"),
    //     };
    //     self.win_state = self.check_win_condition(&value, (x, y))
    // }

    pub fn set_column(&mut self, value: Team, col: usize) -> Result<(), BoardError> {
        if let GameState::Won(_) = self.game_state {
            return Err(BoardError::GameOver);
        }
        if col >= N {
            return Err(BoardError::OutOfBound(col));
        }
        if self.column_is_full(col) {
            return Err(BoardError::ColumnFull(col));
        }

        let lowest_empty_row = self
            .cells
            .slice(s![.., col])
            .indexed_iter()
            .filter_map(|(row, elem)| elem.is_none().then_some(row))
            .last()
            .unwrap();

        // Set value
        self.cells[[lowest_empty_row, col]] = Some(value.clone());

        // Check win condition
        self.game_state = self.update_game_state(&value, (lowest_empty_row, col));

        Ok(())
    }

    fn column_is_full(&self, col: usize) -> bool {
        self.cells
            .slice(s![.., col])
            .iter()
            .all(|elem| elem.is_some())
    }

    fn update_game_state(&self, value: &Team, (x, y): (usize, usize)) -> GameState {
        if Self::_all_equal(self.cells.slice(s![x, ..]), value)
            || Self::_all_equal(self.cells.slice(s![.., y]), value)
            || Self::_all_equal(self.cells.diag(), value)
            || Self::_all_equal(
                // Anti-diagonal
                self.cells
                    .indexed_iter()
                    .filter_map(|((row, col), elem)| ((row + col) == N - 1).then_some(elem)),
                value,
            )
        {
            GameState::Won(value.clone())
        } else if self.cells.iter().any(|elem| elem.is_none()) {
            GameState::NotYetWon
        } else {
            GameState::Stalemate
        }
    }

    #[inline]
    fn _all_equal<'item, T: PartialEq + 'item>(
        array: impl IntoIterator<Item = &'item Option<T>>,
        value: &T,
    ) -> bool {
        array.into_iter().all(|elem| elem.as_ref() == Some(value))
    }
}

impl<const N: usize> Default for Board<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Display for Board<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.cells.rows() {
            // Left wall
            write!(f, "{WALL_ICON}")?;
            for cell in row.iter() {
                let emoji = match cell {
                    Some(Team::Milk) => MILK_ICON,
                    Some(Team::Cookie) => COOKIE_ICON,
                    None => EMPTY_ICON,
                };
                write!(f, "{emoji}")?;
            }

            // Right wall and new line
            writeln!(f, "{WALL_ICON}")?;
        }

        // Bottom wall
        let num_tiles = self.cells.dim().0 + 2;
        writeln!(f, "{}", vec![WALL_ICON; num_tiles].join(""))?;

        // Winner announcement
        match &self.game_state {
            GameState::Won(team) => writeln!(f, "{team} wins!")?,
            GameState::Stalemate => writeln!(f, "No winner.")?,
            GameState::NotYetWon => (),
        };

        Ok(())
    }
}

pub async fn board(State(state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, state.read().await.board.to_string())
}

pub async fn reset(State(state): State<AppState>) -> impl IntoResponse {
    let board = &mut state.write().await.board;
    board.reset();
    (StatusCode::OK, board.to_string())
}

pub async fn place(
    State(state): State<AppState>,
    Path((team, column)): Path<(String, usize)>,
) -> impl IntoResponse {
    let board = &mut state.write().await.board;
    if column == 0 || column > board.size() {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(team) = serde_json::from_str(&format!(r#""{team}""#)) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match (board.set_column(team, column - 1), &board.game_state) {
        // Errors, or the game has just ended (Won or Stalemate)
        (Err(_), _) => (StatusCode::SERVICE_UNAVAILABLE, board.to_string()).into_response(),
        _ => (StatusCode::OK, board.to_string()).into_response(),
    }
}
