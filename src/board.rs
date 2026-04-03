use rand::rng;
use rand::seq::SliceRandom;

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
    fn test_index_calculation(small_board: Board) {
        assert_eq!(small_board.get_index(0, 1), 5);
        assert_eq!(small_board.get_index(4, 0), 4);
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
        assert_eq!(neighbors, vec![6,7,8,11,13,16,17,18]);
    }

    #[rstest]
    fn test_get_neighbors_edge_corner(small_board: Board) {
        let mut neighbors = small_board.get_neighbors_indices(2, 0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1,3,6,7,8]);

        let mut neighbors = small_board.get_neighbors_indices(0, 0);
        neighbors.sort();
        assert_eq!(neighbors, vec![1,5,6]);

        let mut neighbors = small_board.get_neighbors_indices(4, 4);
        neighbors.sort();
        assert_eq!(neighbors, vec![18,19,23]);
        
    }

}
