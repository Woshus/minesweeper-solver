use super::{Board, Cell, CellContent, CellState};
use std::collections::{HashMap, HashSet, VecDeque};
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentResult {
    pub cell_indices: Vec<usize>,
    pub total_ways_per_mine_count: HashMap<usize, usize>,
    pub cell_mine_tallies_by_count: HashMap<usize, Vec<usize>>,
}

type Segment = Vec<usize>;
type CellIndex = usize;
type CellProbability = f32;
type Probabilities = HashMap<CellIndex, CellProbability>;

fn build_binom_table(max_n: usize) -> Vec<Vec<f64>> {
    let mut table = vec![vec![0.0f64; max_n + 1]; max_n + 1];
    for n in 0..=max_n {
        table[n][0] = 1.0;
        for k in 1..=n {
            table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
        }
    }
    table
}

#[inline]
fn binom(table: &[Vec<f64>], n: usize, k: usize) -> f64 {
    if k > n { 0.0 } else { table[n][k] }
}

fn convolve(
    first_poly: &HashMap<usize, f64>,
    second_poly: &HashMap<usize, f64>,
) -> HashMap<usize, f64> {
    let mut result: HashMap<usize, f64> = HashMap::new();
    for (&ka, &wa) in first_poly {
        for (&kb, &wb) in second_poly {
            *result.entry(ka + kb).or_insert(0.0) += wa * wb;
        }
    }
    result
}

fn deconvolve(global: &HashMap<usize, f64>, segment: &HashMap<usize, f64>) -> HashMap<usize, f64> {
    // Find the minimum key in the segment polynomial (its "offset").
    let seg_min = segment.keys().copied().min().unwrap_or(0);
    // Shift segment so it starts at 0.
    let seg_shifted: HashMap<usize, f64> =
        segment.iter().map(|(&k, &v)| (k - seg_min, v)).collect();

    let global_max = global.keys().copied().max().unwrap_or(0);
    let seg_max_shifted = seg_shifted.keys().copied().max().unwrap_or(0);
    let partner_max = global_max.saturating_sub(seg_min);

    let seg0 = *seg_shifted.get(&0).unwrap_or(&0.0);
    assert!(
        seg0 != 0.0,
        "deconvolve: segment polynomial has no constant term after shift"
    );

    let mut partner: HashMap<usize, f64> = HashMap::new();

    for k in 0..=partner_max {
        // global[k + seg_min] = Σ_{j=0}^{min(k, seg_max_shifted)} seg_shifted[j] * partner[k-j]
        let global_k = *global.get(&(k + seg_min)).unwrap_or(&0.0);
        let mut acc = global_k;
        for j in 1..=k.min(seg_max_shifted) {
            if let Some(&sv) = seg_shifted.get(&j) {
                if let Some(&pv) = partner.get(&(k - j)) {
                    acc -= sv * pv;
                }
            }
        }
        let val = acc / seg0;
        if val.abs() > 1e-9 {
            partner.insert(k, val);
        }
    }

    partner
}

impl Board {
    pub fn calculate_probabilities(&self) -> Probabilities {
        let frontier_cells = self.get_frontier_cell_indices();
        let frontier_segments = self.get_isolated_frontier_segments(&frontier_cells);
        let mut probabilities = Probabilities::new();

        // Calculate remaining floating cells
        let floating_cells = self.get_floating_count();
        let mines_remaining = self.get_mines_remaining();

        // Early exit: no mines remaining — all hidden cells are safe
        if mines_remaining == 0 {
            for (idx, cell) in self.cells.iter().enumerate() {
                if matches!(cell.state, CellState::Hidden | CellState::Flagged) {
                    probabilities.insert(idx, 0.0);
                }
            }
            return probabilities;
        }

        // Get segment results via exhaustive enumeration
        let segment_results: Vec<SegmentResult> = frontier_segments
            .iter()
            .map(|seg| self.get_possible_orientations(seg))
            .collect();

        // Build segment polynomials: HashMap<mine_count, ways_as_f64>
        let segment_polys: Vec<HashMap<usize, f64>> = segment_results
            .iter()
            .map(|seg_res| {
                seg_res
                    .total_ways_per_mine_count
                    .iter()
                    .map(|(&mine_count, &ways)| (mine_count, ways as f64))
                    .collect()
            })
            .collect();

        // Convolve all segment polynomials into global_configs
        let mut global_configs: HashMap<usize, f64> = HashMap::new();
        global_configs.insert(0, 1.0);
        for poly in &segment_polys {
            global_configs = convolve(&global_configs, poly);
        }

        // Build binomial table for floating cell combinations
        let binom_table = build_binom_table(floating_cells.max(1));

        // Compute total weight:
        // Σ_k global_configs[k] * C(floating_cells, mines_remaining - k)
        let mut total_weight: f64 = 0.0;
        for (&k, &ways) in &global_configs {
            if k > mines_remaining {
                continue;
            }
            let float_mines = mines_remaining - k;
            if float_mines > floating_cells {
                continue;
            }
            total_weight += ways * binom(&binom_table, floating_cells, float_mines);
        }

        // No valid configurations found
        if total_weight == 0.0 {
            return probabilities;
        }

        // Compute prefix and suffix convolutions for efficient rest-polynomial computation
        let num_segments = segment_polys.len();
        let mut prefix: Vec<HashMap<usize, f64>> = Vec::with_capacity(num_segments + 1);
        prefix.push({
            let mut identity = HashMap::new();
            identity.insert(0, 1.0);
            identity
        });
        for poly in &segment_polys {
            prefix.push(convolve(prefix.last().unwrap(), poly));
        }

        // suffix[i] = convolution of segment_polys[num_segments - i .. num_segments]
        let mut suffix: Vec<HashMap<usize, f64>> = Vec::with_capacity(num_segments + 1);
        suffix.push({
            let mut identity = HashMap::new();
            identity.insert(0, 1.0);
            identity
        });
        for poly in segment_polys.iter().rev() {
            suffix.push(convolve(suffix.last().unwrap(), poly));
        }

        // Marginalize per-cell probabilities for each segment
        for (seg_idx, seg_result) in segment_results.iter().enumerate() {
            // rest = convolution of all segments except seg_idx
            // prefix[seg_idx] = poly[0] * ... * poly[seg_idx-1]
            // suffix[num_segments - 1 - seg_idx] = poly[num_segments-1] * ... * poly[seg_idx+1]
            let rest = convolve(&prefix[seg_idx], &suffix[num_segments - 1 - seg_idx]);

            for (pos, &cell_idx) in seg_result.cell_indices.iter().enumerate() {
                let mut cell_numerator: f64 = 0.0;

                for (&seg_mines, tally_row) in &seg_result.cell_mine_tallies_by_count {
                    let cell_mine_configs = tally_row[pos] as f64;
                    if cell_mine_configs == 0.0 {
                        continue;
                    }

                    for (&rest_mines, &rest_weight) in &rest {
                        let total_seg_and_rest = seg_mines + rest_mines;
                        if total_seg_and_rest > mines_remaining {
                            continue;
                        }
                        let float_mines = mines_remaining - total_seg_and_rest;
                        if float_mines > floating_cells {
                            continue;
                        }

                        cell_numerator += cell_mine_configs
                            * rest_weight
                            * binom(&binom_table, floating_cells, float_mines);
                    }
                }

                let prob = (cell_numerator / total_weight) as f32;
                probabilities.insert(cell_idx, prob);
            }
        }

        // Compute floating cell probabilities
        let floating_indices: Vec<usize> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| matches!(cell.state, CellState::Hidden))
            .filter(|(cell_idx, _)| {
                self.get_neighbors_indices(*cell_idx)
                    .iter()
                    .all(|neighbor_idx| {
                        matches!(self.cells[*neighbor_idx].state, CellState::Hidden)
                    })
            })
            .map(|(idx, _)| idx)
            .collect();

        for &float_idx in &floating_indices {
            let mut float_numerator: f64 = 0.0;

            for (&k, &ways) in &global_configs {
                if k > mines_remaining {
                    continue;
                }
                let float_mines = mines_remaining - k;
                if float_mines == 0 || float_mines > floating_cells {
                    continue;
                }

                // Ways for this specific floating cell to be a mine:
                // segment configs * C(floating_cells - 1, float_mines - 1)
                float_numerator += ways * binom(&binom_table, floating_cells - 1, float_mines - 1);
            }

            let prob = (float_numerator / total_weight) as f32;
            probabilities.insert(float_idx, prob);
        }

        probabilities
    }

    fn get_floating_count(&self) -> usize {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| matches!(cell.state, CellState::Hidden))
            .filter(|(cell_idx, _)| {
                self.get_neighbors_indices(*cell_idx)
                    .iter()
                    .all(|neighbor_idx| {
                        matches!(self.cells[*neighbor_idx].state, CellState::Hidden)
                    })
            })
            .count()
    }

    // Only returns revealed neighbors that are numbers
    // Note: This should only be called with the idx of a hidden cell. At least with its current usage in
    // finding probabilities.
    fn get_adjacent_revealed_numbers(&self, idx: usize) -> Vec<usize> {
        self.get_neighbors_indices(idx)
            .into_iter()
            .filter(|&neighbor_idx| {
                let cell = &self.cells[neighbor_idx];
                cell.state == CellState::Revealed && matches!(cell.content, CellContent::Number(_))
            })
            .collect()
    }

    /*
       Returns a struct containing the original segment, and its possible
       legal configuration counts for each cell.
    */
    #[allow(dead_code)]
    fn get_possible_orientations(&self, segment: &Vec<usize>) -> SegmentResult {
        // SegmentResult contains all necessary information needed to calculate cell
        // probabilities for later.

        let mut result = SegmentResult {
            cell_indices: segment.clone(),
            total_ways_per_mine_count: HashMap::new(),
            cell_mine_tallies_by_count: HashMap::new(),
        };

        // contains all clues adjacent to the current segment, set to prevent duplicates
        let mut relevant_clues = HashSet::new();
        for &cell_idx in segment {
            for neighbor in self.get_adjacent_revealed_numbers(cell_idx) {
                relevant_clues.insert(neighbor);
            }
        }

        // transform clues to be readily used
        let relevant_clues: Vec<(usize, i32)> = relevant_clues
            .into_iter()
            .map(|idx| {
                if let CellContent::Number(num) = self.cells[idx].content {
                    (idx, num as i32)
                } else {
                    panic!("Clue at {} is not a number", idx);
                }
            })
            .collect();

        let mut current_config = vec![false; segment.len()];

        fn rec_get_orientations(
            board: &Board,
            config_idx: usize,
            config: &mut Vec<bool>,
            segment: &Vec<usize>,
            clues: &Vec<(usize, i32)>,
            result: &mut SegmentResult,
        ) {
            for &(clue_idx, clue_value) in clues {
                let neighbors = board.get_neighbors_indices(clue_idx);
                let mut confirmed_mines = 0;
                let mut undecided_hidden = 0;

                for n_idx in neighbors {
                    if let Some(pos_in_seg) = segment.iter().position(|&x| x == n_idx) {
                        if pos_in_seg < config_idx {
                            if config[pos_in_seg] {
                                confirmed_mines += 1;
                            }
                        } else {
                            undecided_hidden += 1;
                        }
                    }
                }

                if confirmed_mines > clue_value || confirmed_mines + undecided_hidden < clue_value {
                    return;
                }
            }

            if config_idx == segment.len() {
                let mine_count = config.iter().filter(|&&m| m).count();
                *result
                    .total_ways_per_mine_count
                    .entry(mine_count)
                    .or_insert(0) += 1;
                let tallies = result
                    .cell_mine_tallies_by_count
                    .entry(mine_count)
                    .or_insert(vec![0; segment.len()]);
                for (i, &is_mine) in config.iter().enumerate() {
                    if is_mine {
                        tallies[i] += 1;
                    }
                }
                return;
            }
            config[config_idx] = false;
            rec_get_orientations(board, config_idx + 1, config, segment, clues, result);

            config[config_idx] = true;
            rec_get_orientations(board, config_idx + 1, config, segment, clues, result);
        }

        rec_get_orientations(
            self,
            0,
            &mut current_config,
            segment,
            &relevant_clues,
            &mut result,
        );
        result
    }

    // This returns a list of cells to be used in probability calculation.
    // Note: Connected in this context means that the cells share at least 1 revealed
    // cell with a number in it.
    // Note: This might seem roundabout but it prevents overlap of "unconnected"
    // cells. refer to test_get_frontier_multiple_segment()
    #[allow(dead_code)]
    fn get_connected_frontier_cells(&self, cell_idx: usize, frontier: &[usize]) -> Vec<usize> {
        let mut connected = Vec::new();
        let clues = self.get_adjacent_revealed_numbers(cell_idx);

        for clue_idx in clues {
            let neighbors_of_clue = self.get_neighbors_indices(clue_idx);
            for neighbor_idx in neighbors_of_clue {
                if frontier.contains(&neighbor_idx) && neighbor_idx != cell_idx {
                    connected.push(neighbor_idx);
                }
            }
        }
        connected
    }

    // This function returns every hidden cell (includes flagged cells) that has a revealed number attached to it
    #[allow(dead_code)]
    fn get_frontier_cell_indices(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter(|(_, cell)| {
                matches!(cell.state, CellState::Hidden) | matches!(cell.state, CellState::Flagged)
            })
            .filter(|(idx, _)| self.is_touching_revealed_number(*idx))
            .map(|(idx, _)| idx)
            .collect()
    }

    // Helper for get_frontier_cell_indices
    fn is_touching_revealed_number(&self, idx: usize) -> bool {
        self.get_neighbors_indices(idx).iter().any(|&neighbor_idx| {
            let neighbor = &self.cells[neighbor_idx];
            matches!(
                (neighbor.state, neighbor.content),
                (CellState::Revealed, CellContent::Number(_))
            )
        })
    }

    // This function separates a list of cells into segments that are separate probabilistic regions.
    #[allow(dead_code)]
    fn get_isolated_frontier_segments(&self, frontier_cells: &[usize]) -> Vec<Segment> {
        let mut segments = Vec::new();
        let mut visited = HashSet::new();

        for &start_node in frontier_cells {
            if visited.contains(&start_node) {
                continue;
            }

            let mut current_segment = Vec::new();
            let mut queue = VecDeque::from([start_node]);
            visited.insert(start_node);

            while let Some(cell_idx) = queue.pop_front() {
                current_segment.push(cell_idx);

                for neighbor in self.get_connected_frontier_cells(cell_idx, frontier_cells) {
                    if !visited.contains(&neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
            segments.push(current_segment);
        }

        segments
    }

    fn get_mines_remaining(&self) -> usize {
        let flagged = self
            .cells
            .iter()
            .filter(|c| c.state == CellState::Flagged)
            .count();
        self.mines.saturating_sub(flagged)
    }

    #[cfg(test)]
    pub(crate) fn from_fields(width: usize, height: usize, mines: usize, cells: Vec<Cell>) -> Self {
        Self {
            width,
            height,
            mines,
            cells,
        }
    }

    #[cfg(test)]
    pub(crate) fn set_state(&mut self, idx: usize, state: CellState) {
        self.cells[idx].state = state;
    }

    #[cfg(test)]
    pub(crate) fn set_content(&mut self, idx: usize, content: CellContent) {
        self.cells[idx].content = content;
    }

    #[cfg(test)]
    pub(crate) fn place_number_on_cell(&mut self, idx: usize) {
        if let CellContent::Number(ref mut num) = self.cells[idx].content {
            *num += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn build_board_with_mines(width: usize, height: usize, mine_indices: &[usize]) -> Board {
        let mut board = Board::new(width, height, 0);
        for &idx in mine_indices {
            board.set_content(idx, CellContent::Mine);
        }
        for &idx in mine_indices {
            for neighbor_idx in board.get_neighbors_indices(idx) {
                board.place_number_on_cell(neighbor_idx);
            }
        }
        board
    }

    #[test]
    fn test_get_frontier_cells() {
        let mut board = build_board_with_mines(5, 5, &[4, 8, 12, 16, 20]);
        board.click_cell(0);
        let frontier_cells = board.get_frontier_cell_indices();
        assert_eq!(frontier_cells, vec![3, 8, 12, 13, 15, 16, 17]);
    }

    #[test]
    fn test_get_frontier_segments() {
        let mut board = build_board_with_mines(5, 5, &[4, 8, 12, 16, 20]);
        board.click_cell(0);
        let frontier_cells = board.get_frontier_cell_indices();
        let frontier_segments = board.get_isolated_frontier_segments(&frontier_cells);
        assert_eq!(frontier_segments, [vec![3, 8, 12, 13, 15, 16, 17]]);

        board.click_cell(24);
        let frontier_cells = board.get_frontier_cell_indices();
        let mut frontier_segments = board.get_isolated_frontier_segments(&frontier_cells);
        frontier_segments
            .iter_mut()
            .for_each(|segment| segment.sort());
        assert_eq!(frontier_segments, [vec![3, 8, 9, 12, 15, 16, 21]]);
    }

    #[test]
    fn test_get_frontier_multiple_segments() {
        let mut board = Board::new(10, 10, 0);
        let mine_placements = [2, 3, 7, 20, 22, 27, 29, 30, 33];
        for mine_idx in mine_placements {
            board.set_content(mine_idx, CellContent::Mine);
        }
        for mine_idx in mine_placements {
            for neighbor_idx in board.get_neighbors_indices(mine_idx) {
                board.place_number_on_cell(neighbor_idx);
            }
        }
        board.click_cell(0);
        let mut frontier_cells = board.get_frontier_cell_indices();
        frontier_cells.sort();
        assert_eq!(frontier_cells, [2, 12, 20, 21, 22]);

        let mut frontier_segments = board.get_isolated_frontier_segments(&frontier_cells);
        frontier_segments
            .iter_mut()
            .for_each(|segment| segment.sort());
        assert_eq!(frontier_segments, [[2, 12, 20, 21, 22]]);

        board.click_cell(99);
        let mut frontier_cells = board.get_frontier_cell_indices();
        frontier_cells.sort();
        assert_eq!(
            frontier_cells,
            [
                2, 3, 7, 12, 13, 17, 20, 21, 22, 23, 27, 28, 29, 30, 31, 32, 33
            ]
        );

        let mut frontier_segments = board.get_isolated_frontier_segments(&frontier_cells);
        frontier_segments
            .iter_mut()
            .for_each(|segment| segment.sort());

        assert_eq!(
            frontier_segments,
            [
                vec![2, 12, 20, 21, 22],
                vec![3, 13, 23, 30, 31, 32, 33],
                vec![7, 17, 27, 28, 29]
            ]
        );

        board.click_cell(32);
        let frontier_cells = board.get_frontier_cell_indices();
        let mut frontier_segments = board.get_isolated_frontier_segments(&frontier_cells);
        frontier_segments
            .iter_mut()
            .for_each(|segment| segment.sort());

        assert_eq!(
            frontier_segments,
            [
                vec![2, 3, 12, 13, 20, 21, 22, 23, 30, 31, 33],
                vec![7, 17, 27, 28, 29]
            ]
        );
    }

    #[test]
    fn test_floating_cell_count() {
        let mut board = build_board_with_mines(5, 5, &[4, 8, 12, 16, 20]);
        board.click_cell(0);
        let floating_count = board.get_floating_count();
        assert_eq!(floating_count, 10);
    }

    #[test]
    fn test_get_possible_orientations() {
        let mut board = build_board_with_mines(5, 5, &[4, 8, 12, 16, 20]);
        board.click_cell(0);
        let frontier_cells = board.get_frontier_cell_indices();
        let segments = board.get_isolated_frontier_segments(&frontier_cells);
        let orientations = board.get_possible_orientations(&segments[0]);
        assert_eq!(
            orientations.total_ways_per_mine_count,
            HashMap::from([(3, 4)])
        );
        assert_eq!(
            orientations.cell_mine_tallies_by_count,
            HashMap::from([(3, vec![2, 2, 4, 0, 2, 2, 0])])
        );
    }

    #[test]
    fn test_get_possible_orientations_mult() {
        let mut board = Board::new(10, 10, 0);
        let mine_placements = [2, 3, 7, 15, 20, 22, 27, 29, 30, 33];
        for mine_idx in mine_placements {
            board.set_content(mine_idx, CellContent::Mine);
        }
        for mine_idx in mine_placements {
            for neighbor_idx in board.get_neighbors_indices(mine_idx) {
                board.place_number_on_cell(neighbor_idx);
            }
        }

        board.click_cell(99);
        let frontier_cells = board.get_frontier_cell_indices();
        let segments = board.get_isolated_frontier_segments(&frontier_cells);
        let second_segment_orientations = board.get_possible_orientations(&segments[0]);

        assert_eq!(
            second_segment_orientations.total_ways_per_mine_count,
            HashMap::from([(6, 2), (5, 1)])
        );

        assert_eq!(
            second_segment_orientations.cell_mine_tallies_by_count,
            HashMap::from([
                (6, vec![1, 0, 2, 1, 0, 0, 0, 1, 1, 2, 2, 0, 2]),
                (5, vec![0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1])
            ])
        );
    }

    #[test]
    fn test_probability_calculation() {
        let mut board = Board::new(10, 10, 0);
        let mine_placements = [2, 3, 7, 15, 20, 22, 27, 29, 30, 33];
        for mine_idx in mine_placements {
            board.set_content(mine_idx, CellContent::Mine);
        }
        for mine_idx in mine_placements {
            for neighbor_idx in board.get_neighbors_indices(mine_idx) {
                board.place_number_on_cell(neighbor_idx);
            }
        }
        board.click_cell(0);
        board.click_cell(99);
        board.calculate_probabilities();
    }
}
