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
    pub(crate) content: CellContent,
    pub(crate) state: CellState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    width: usize,
    height: usize,
    mines: usize,
    cells: Vec<Cell>,
}

impl Board {
    /// Creates a new Minesweeper board with the speicified dimensions.
    ///
    /// All cells are initialized as `Hidden`
    ///
    ///  # Panics
    ///
    /// Panics if `width` or `height` is 0.
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "Dimensions must be greater than zero"
        );
        assert!(
            mines < width * height,
            "Mines should be less than total squares"
        );
        let total_cells = width * height;

        Self {
            width,
            height,
            mines,
            cells: vec![
                Cell {
                    content: CellContent::Number(0),
                    state: CellState::Hidden,
                };
                total_cells
            ],
        }
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

    /// Converts a 2D coordinate (x, y) into a 1D linear index.
    pub fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Converts a 1D linear index into 2D coordinates (x,y).
    pub fn get_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }

    pub fn get_width(&self) -> usize {
        self.width
    }
    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_mine_count(&self) -> usize {
        self.mines
    }

    pub fn cell_iter(&self) -> impl Iterator<Item = (CellContent, CellState)> {
        self.cells.iter().map(|cell| (cell.content, cell.state))
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

pub mod gameplay;
pub mod generation;
pub mod probabilities;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn small_board() -> Board {
        Board::new(5, 5, 0)
    }

    #[rstest]
    fn test_board_vec_size(small_board: Board) {
        assert_eq!(small_board.cells.len(), 25);
    }

    #[rstest]
    fn test_index_and_coords_calculation(small_board: Board) {
        assert_eq!(small_board.get_index(0, 1), 5);
        assert_eq!(small_board.get_index(4, 0), 4);
        assert_eq!(small_board.get_coords(5), (0, 1));
        assert_eq!(small_board.get_coords(4), (4, 0));
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
}
