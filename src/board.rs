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
    pub content: CellContent,
    pub state: CellState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
}

impl Board {
    // pub fn new(width: usize, height: usize, mine_count: usize) -> Self {
    //     Self {}
    // }

    pub fn new_empty(width: usize, height: usize) -> Self {
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

    fn get_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn get_coords(&self, idx: usize) -> (usize, usize) {
        (idx % self.width, idx / self.width)
    }

    fn get_neighbors_indices(&self, x: usize, y: usize) -> Vec<usize> {
        let mut neighbors = Vec::with_capacity(8);

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

    pub fn generate_mines(&mut self, start_x: usize, start_y: usize, total_mines: usize) {
        let mut rng = rng();
        let start_idx = self.get_index(start_x, start_y);

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

        let (x, y) = self.get_coords(idx);
        for neighbor_idx in self.get_neighbors_indices(x, y) {
            if let CellContent::Number(ref mut count) = self.cells[neighbor_idx].content {
                *count += 1;
            }
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.get_index(x, y);
                match self.cells[idx].content {
                    CellContent::Mine => write!(f, "● ")?,
                    CellContent::Number(n) => write!(f, "{} ", n)?,
                };
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
        Board::new_empty(5, 5)
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
        let mut neighbors = small_board.get_neighbors_indices(2, 2);
        neighbors.sort();
        assert_eq!(neighbors, vec![6, 7, 8, 11, 13, 16, 17, 18]);
    }

    #[rstest]
    fn test_get_neighbors_edge_corner(small_board: Board) {
        let mut neighbors = small_board.get_neighbors_indices(2, 0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 3, 6, 7, 8]);

        let mut neighbors = small_board.get_neighbors_indices(0, 0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1, 5, 6]);

        let mut neighbors = small_board.get_neighbors_indices(4, 4);
        neighbors.sort();
        assert_eq!(neighbors, vec![18, 19, 23]);
    }

    #[test]
    fn test_mine_placement() {
        let mut board = Board::new_empty(5, 5);
        board.place_mine(12);

        let (x, y) = board.get_coords(12);
        board.get_neighbors_indices(x, y).iter().for_each(|&idx| {
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
        let mut board = Board::new_empty(5, 5);
        board.place_mine(12);
        board.place_mine(12);

        let (x, y) = board.get_coords(12);
        board.get_neighbors_indices(x, y).iter().for_each(|&idx| {
            assert_eq!(CellContent::Number(1), board.cells[idx].content);
        });
    }
}
