use super::{Board, CellContent};
use rand::{rng, seq::SliceRandom};

impl Board {
    pub fn generate_mines(&mut self, start_idx: usize) {
        let mut rng = rng();

        let mut indices: Vec<usize> = Vec::with_capacity(self.cells.len() - 1);
        indices.extend(0..start_idx);
        indices.extend((start_idx + 1)..self.cells.len());

        indices.shuffle(&mut rng);

        indices
            .iter()
            .take(self.mines)
            .for_each(|&idx| self.place_mine(idx));
    }

    pub(crate) fn place_mine(&mut self, idx: usize) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mine_placement() {
        let mut board = Board::new(5, 5, 0);
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
        let mut board = Board::new(5, 5, 0);
        board.place_mine(12);
        board.place_mine(12);

        board.get_neighbors_indices(12).iter().for_each(|&idx| {
            assert_eq!(CellContent::Number(1), board.cells[idx].content);
        });
    }
}
