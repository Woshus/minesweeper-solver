use super::{Board, CellContent, CellState};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RevealResult {
    HitMine,
    Opened,
    Chorded,
    NoOp,
}

impl Board {
    /// Checks whether the board has any `Hidden` cells left that aren't mines.
    pub fn is_win_condition_met(&self) -> bool {
        self.cells
            .iter()
            .filter(|cell| !matches!(cell.content, CellContent::Mine))
            .all(|cell| matches!(cell.state, CellState::Revealed))
    }

    pub fn toggle_flag(&mut self, idx: usize) {
        if self.cells[idx].state == CellState::Hidden {
            self.cells[idx].state = CellState::Flagged;
        } else if self.cells[idx].state == CellState::Flagged {
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

    pub fn reveal_all_mines(&mut self) {
        self.cells
            .iter_mut()
            .filter(|cell| cell.content == CellContent::Mine)
            .for_each(|cell| cell.state = CellState::Revealed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    fn place_mine_for_test(board: &mut Board, idx: usize) {
        if matches!(board.cells[idx].content, CellContent::Mine) {
            return;
        }

        board.cells[idx].content = CellContent::Mine;

        for neighbor_idx in board.get_neighbors_indices(idx) {
            if let CellContent::Number(ref mut count) = board.cells[neighbor_idx].content {
                *count += 1;
            }
        }
    }

    #[fixture]
    fn board_with_diagonal_mines() -> Board {
        let mut board = Board::new(5, 5, 0);
        for mine_idx in [4, 8, 12, 16, 20] {
            place_mine_for_test(&mut board, mine_idx);
        }
        board
    }

    #[rstest]
    fn test_click_multiple_cells(mut board_with_diagonal_mines: Board) {
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
        board_with_diagonal_mines.toggle_flag(12);
        assert_eq!(
            board_with_diagonal_mines.cells[12].state,
            CellState::Flagged
        );
        board_with_diagonal_mines.toggle_flag(12);
        assert_eq!(board_with_diagonal_mines.cells[12].state, CellState::Hidden);
    }

    #[rstest]
    fn test_chord_cell_fails(mut board_with_diagonal_mines: Board) {
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);
        board_with_diagonal_mines.click_cell(7);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        board_with_diagonal_mines.toggle_flag(8);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        board_with_diagonal_mines.toggle_flag(2);
        board_with_diagonal_mines.toggle_flag(3);
        assert_eq!(board_with_diagonal_mines.chord_cell(7), RevealResult::NoOp);

        board_with_diagonal_mines.toggle_flag(3);
        assert_eq!(
            board_with_diagonal_mines.chord_cell(7),
            RevealResult::HitMine
        );
    }

    #[rstest]
    fn test_chord_cell_success(mut board_with_diagonal_mines: Board) {
        board_with_diagonal_mines.click_cell(7);
        board_with_diagonal_mines.toggle_flag(8);
        board_with_diagonal_mines.toggle_flag(12);
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
