//! # Minesweeper Board Engine
//!
//! This module provides the core logic for generating and managing a Minesweeper board.

use rand::rng;
use rand::seq::SliceRandom;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellContent {
    Mine,
    Number(u8),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CellState {
    Hidden,
    Flagged,
    Revealed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cell {
    content: CellContent,
    state: CellState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

// TODO: Below NoOp's can be modified into actual functionality
// - chord_cell() NoOp when not enough cells are flagged can highlight the cells
// around it instead of NoOp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RevealResult {
    HitMine,
    Opened,
    Chorded,
    NoOp,
}

impl Board {
    /// Creates a new Minesweeper board with the speicified dimensions.
    ///
    /// All cells are initialized as `Hidden`
    ///
    ///  # Panics
    ///
    /// Panics if `width` or `height` is 0.
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "Dimensions must be greater than zero"
        );
        let total_cells = width * height;

        Self {
            width,
            height,
            cells: vec![
                Cell {
                    content: CellContent::Number(0),
                    state: CellState::Hidden,
                };
                total_cells
            ],
        }
    }

    #[allow(dead_code)]
    fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = self.get_index(x, y);
        Some(&self.cells[index])
    }

    fn get_neighbors_indices(&self, idx: usize) -> Vec<usize> {
        let mut neighbors = Vec::with_capacity(8);

        let (x, y) = self.get_coords(idx);
        for x_offset in -1..=1 {
            for y_offset in -1..=1 {
                if x_offset == 0 && y_offset == 0 {
                    continue;
                }

                let neighbor_x = x as i32 + x_offset;
                let neighbor_y = y as i32 + y_offset;
                if neighbor_x >= 0
                    && neighbor_x < self.width as i32
                    && neighbor_y >= 0
                    && neighbor_y < self.height as i32
                {
                    let index = self.get_index(neighbor_x as usize, neighbor_y as usize);
                    neighbors.push(index);
                }
            }
        }
        neighbors
    }

    /// Randomly distributes a set number of mines across the board.
    ///
    /// This function also increments the value stored in all cells
    /// adjacent to the newly placed mines.
    ///
    /// # Panics
    ///
    /// Panics if `total_mines` is greater than the total number of cells.
    ///
    // TODO: Currently the start of a game can generate boards that start with CellContent::Number(1).
    // Consider adding guaranteed opening logic.
    pub fn generate_mines(&mut self, start_idx: usize, total_mines: usize) {
        assert!(
            total_mines <= self.cells.len(),
            "Cannot place more mines than cells available"
        );
        let mut rng = rng();

        let mut indices: Vec<usize> = Vec::with_capacity(self.cells.len() - 1);
        indices.extend(0..start_idx);
        indices.extend((start_idx + 1)..self.cells.len());

        indices.shuffle(&mut rng);

        indices
            .iter()
            .take(total_mines)
            .for_each(|&idx| self.place_mine(idx));
    }

    fn place_mine(&mut self, idx: usize) {
        if matches!(self.cells[idx].content, CellContent::Mine) {
            return;
        }

        self.cells[idx].content = CellContent::Mine;

        for neighbor_idx in self.get_neighbors_indices(idx) {
            if let CellContent::Number(ref mut count) = self.cells[neighbor_idx].content {
                *count += 1;
            }
        }
    }

    /// Places a flag on a specified `Hidden` cell.
    pub fn place_flag(&mut self, idx: usize) {
        if self.cells[idx].state == CellState::Hidden {
            self.cells[idx].state = CellState::Flagged;
        }
    }

    /// Removes a flag on a specified `Flagged` cell.
    pub fn remove_flag(&mut self, idx: usize) {
        if self.cells[idx].state == CellState::Flagged {
            self.cells[idx].state = CellState::Hidden;
        }
    }

    /// This function handles the logic of what happens when a cell is clicked/revealed.
    ///
    /// The return value is an `RevealResult` enum, which is used by the TBD game engine
    /// to handle game events.
    pub fn click_cell(&mut self, idx: usize) -> RevealResult {
        assert!(idx < self.cells.len());
        if self.cells[idx].state == CellState::Flagged {
            return RevealResult::NoOp;
        }

        // Logic for hitting a bomb will be handled in game engine
        if self.cells[idx].content == CellContent::Mine {
            return RevealResult::HitMine;
        }

        if self.cells[idx].state == CellState::Revealed
            && let CellContent::Number(num) = self.cells[idx].content
        {
            if num == 0 {
                return RevealResult::NoOp;
            } else {
                return self.chord_cell(idx);
            }
        }

        self.cascade_open(idx);
        RevealResult::Opened
    }

    fn chord_cell(&mut self, idx: usize) -> RevealResult {
        let CellContent::Number(num) = self.cells[idx].content else {
            return RevealResult::NoOp;
        };

        let neighbors = self.get_neighbors_indices(idx);

        let neighbor_flag_count = neighbors
            .iter()
            .filter(|&&idx| self.cells[idx].state == CellState::Flagged)
            .count() as u8;

        // TODO: Instead of NoOp, potentially highlight cells instead, while
        // MouseButton down or similar.
        if neighbor_flag_count != num {
            return RevealResult::NoOp;
        }

        if neighbors.iter().any(|&idx| {
            self.cells[idx].content == CellContent::Mine
                && self.cells[idx].state != CellState::Flagged
        }) {
            return RevealResult::HitMine;
        }

        for neighbor_idx in neighbors {
            if self.cells[neighbor_idx].state == CellState::Hidden {
                self.cascade_open(neighbor_idx);
            }
        }
        RevealResult::Chorded
    }

    // Performs an iterative DFS to reveal connected empty cells and their neighbors.
    // This uses a Vec to avoid stack overflow on large boards.
    fn cascade_open(&mut self, idx: usize) {
        let mut to_visit = vec![idx];

        while let Some(idx) = to_visit.pop() {
            if self.cells[idx].state == CellState::Revealed {
                continue;
            }
            if let CellContent::Number(num) = self.cells[idx].content {
                self.cells[idx].state = CellState::Revealed;
                if num == 0 {
                    for neighbor_idx in self.get_neighbors_indices(idx) {
                        if self.cells[neighbor_idx].state != CellState::Revealed {
                            to_visit.push(neighbor_idx);
                        }
                    }
                }
            }
        }
    }

    /// Converts a 2D coordinate (x, y) into a 1D linear index.
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Converts a 1D linear index into 2D coordinates (x,y).
    pub fn get_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                match self.cells[idx].state {
                    CellState::Flagged => write!(f, "🚩")?,
                    CellState::Hidden => write!(f, "■ ")?,
                    CellState::Revealed => {
                        match self.cells[idx].content {
                            CellContent::Mine => write!(f, "● ")?,
                            CellContent::Number(n) => write!(f, "{} ", n)?,
                        };
                    }
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

mod test {
    use super::*;
    use rstest::*;

    #[fixture]
    fn small_board() -> Board {
        Board::new(5, 5)
    }

    #[fixture]
    fn board_with_diagonal_mines() -> Board {
        let mut board = Board::new(5, 5);
        board.place_mine(4);
        board.place_mine(8);
        board.place_mine(12);
        board.place_mine(16);
        board.place_mine(20);
        board
    }

    #[rstest]
    fn test_board_vec_size(small_board: Board) {
        assert_eq!(small_board.cells.len(), 25)
    }

    #[rstest]
    fn test_index_and_coords_calculation(small_board: Board) {
        assert_eq!(small_board.get_index(0, 1), 5);
        assert_eq!(small_board.get_index(4, 0), 4);
        assert_eq!(small_board.get_coords(5), (0, 1));
        assert_eq!(small_board.get_coords(4), (4, 0));
    }

    #[rstest]
    fn test_invalid_cell(small_board: Board) {
        assert_eq!(small_board.get_cell(7, 7), None);
        assert_eq!(small_board.get_cell(5, 5), None);
    }

    #[rstest]
    fn test_valid_cell(small_board: Board) {
        assert_eq!(
            small_board.get_cell(4, 4),
            Some(&Cell {
                content: CellContent::Number(0),
                state: CellState::Hidden
            })
        );
    }

    #[rstest]
    fn test_get_neighbors(small_board: Board) {
        let mut neighbors = small_board.get_neighbors_indices(12);
        neighbors.sort();
        assert_eq!(neighbors, vec![6, 7, 8, 11, 13, 16, 17, 18]);
    }

    #[rstest]
    fn test_get_neighbors_edge_corner(small_board: Board) {
        let mut neighbors = small_board.get_neighbors_indices(2);
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 3, 6, 7, 8]);

        let mut neighbors = small_board.get_neighbors_indices(0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 5, 6]);

        let mut neighbors = small_board.get_neighbors_indices(24);
        neighbors.sort();
        assert_eq!(neighbors, vec![18, 19, 23]);
    }

    #[test]
    fn test_mine_placement() {
        let mut board = Board::new(5, 5);
        board.place_mine(12);

        board.get_neighbors_indices(12).iter().for_each(|&idx| {
            assert_eq!(CellContent::Number(1), board.cells[idx].content);
        });

        board.place_mine(6);
        board.place_mine(8);
        board.place_mine(16);
        board.place_mine(18);

        assert_eq!(CellContent::Number(3), board.cells[7].content);
        assert_eq!(CellContent::Number(2), board.cells[2].content);
        assert_eq!(CellContent::Number(1), board.cells[1].content);
        assert_eq!(CellContent::Number(1), board.cells[19].content);
    }

    #[test]
    fn test_double_placement_idempotency() {
        let mut board = Board::new(5, 5);
        board.place_mine(12);
        board.place_mine(12);
        

        board.get_neighbors_indices(12).iter().for_each(|&idx| {
            assert_eq!(CellContent::Number(1), board.cells[idx].content);
        });
    }

    #[rstest]
    fn test_click_multiple_cells(mut board_with_diagonal_mines: Board) {
        // TODO: Add test for flags when flagging is implemented
        assert_eq!(
            board_with_diagonal_mines.click_cell(4),
            RevealResult::HitMine
        );
        assert_eq!(board_with_diagonal_mines.cells[4].state, CellState::Hidden);

        assert_eq!(
            board_with_diagonal_mines.click_cell(0),
            RevealResult::Opened
        );
        assert_eq!(board_with_diagonal_mines.click_cell(0), RevealResult::NoOp);
        assert_eq!(board_with_diagonal_mines.click_cell(2), RevealResult::NoOp);
        assert_eq!(
            board_with_diagonal_mines.cells[0].state,
            CellState::Revealed
        );
        assert_eq!(
            board_with_diagonal_mines.cells[2].state,
            CellState::Revealed
        );

        assert_eq!(
            board_with_diagonal_mines.click_cell(9),
            RevealResult::Opened
        );
        assert_eq!(
            board_with_diagonal_mines.cells[9].state,
            CellState::Revealed
        );
        assert_eq!(board_with_diagonal_mines.cells[14].state, CellState::Hidden);
    }

    #[rstest]
    fn test_flagging_and_unflagging(mut board_with_diagonal_mines: Board) {
        board_with_diagonal_mines.click_cell(0);
        assert_eq!(board_with_diagonal_mines.cells[12].state, CellState::Hidden);
        board_with_diagonal_mines.place_flag(12);
        assert_eq!(
            board_with_diagonal_mines.cells[12].state,
            CellState::Flagged
        );
        board_with_diagonal_mines.remove_flag(12);
        assert_eq!(board_with_diagonal_mines.cells[12].state, CellState::Hidden);
    }

    #[rstest]
    fn test_chord_cell_fails(mut board_with_diagonal_mines: Board) {
        // No Flags
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);
        board_with_diagonal_mines.click_cell(7);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        // Too Little / Too Many Flags
        board_with_diagonal_mines.place_flag(8);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        board_with_diagonal_mines.place_flag(2);
        board_with_diagonal_mines.place_flag(3);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        // Wrong Flags
        board_with_diagonal_mines.remove_flag(3);
        assert_eq!(
            board_with_diagonal_mines.chord_cell(7),
            RevealResult::HitMine
        );
    }

    #[rstest]
    fn test_chord_cell_success(mut board_with_diagonal_mines: Board) {
        board_with_diagonal_mines.click_cell(7);
        board_with_diagonal_mines.place_flag(8);
        board_with_diagonal_mines.place_flag(12);
        assert_eq!(
            board_with_diagonal_mines.chord_cell(7),
            RevealResult::Chorded
        );
        assert_eq!(
            board_with_diagonal_mines.cells[0].state,
            CellState::Revealed
        );
        assert_eq!(
            board_with_diagonal_mines.cells[3].state,
            CellState::Revealed
        );
        assert_eq!(
            board_with_diagonal_mines.cells[13].state,
            CellState::Revealed
        );
        assert_eq!(board_with_diagonal_mines.cells[24].state, CellState::Hidden);
    }
}
