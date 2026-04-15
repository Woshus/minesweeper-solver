use crate::board::{Board, CellContent, CellState, RevealResult};

type Dimensions = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Created,
    Playing,
    Won,
    Lost,
}

pub enum CellDisplay {
    Number(u8),
    Mine,
    Flag,
    Hidden,
    Empty,
}

pub struct MinesweeperGame {
    board: Board,
    status: GameStatus,
}

impl MinesweeperGame {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self {
            board: Board::new(width, height, mines),
            status: GameStatus::Created,
        }
    }

    pub fn get_dimensions(&self) -> Dimensions {
        (self.board.get_width(), self.board.get_height())
    }

    pub fn get_mine_count(&self) -> usize {
        self.board.get_mine_count()
    }

    // Returns an iterator of what should be displayed on the board at any given time.
    // Up to the UI for what exactly to display based on what should be displayed.
    pub fn get_display_iter(&self) -> impl Iterator<Item = CellDisplay> {
        self.board.cell_iter().map(|(content, state)| match state {
            CellState::Hidden => CellDisplay::Hidden,
            CellState::Flagged => CellDisplay::Flag,
            _ => match content {
                CellContent::Mine => CellDisplay::Mine,
                CellContent::Number(0) => CellDisplay::Empty,
                CellContent::Number(num) => CellDisplay::Number(num),
            },
        })
    }

    // TODO: if GameStatus::Lost isn't handled, clicking mines does nothing
    pub fn handle_click(&mut self, x: usize, y: usize) {
        // generate mines on first click
        if matches!(self.status, GameStatus::Created) {
            self.board.generate_mines(self.board.get_index(x, y));
            self.status = GameStatus::Playing
        }

        if self.board.click_cell(self.board.get_index(x, y)) == RevealResult::HitMine {
            self.status = GameStatus::Lost;
            self.board.reveal_all_mines();
        }
    }

    pub fn toggle_flag(&mut self, x: usize, y: usize) {
        self.board.toggle_flag(self.board.get_index(x, y));
    }
    pub fn check_win(&mut self) {
        if matches!(self.status, GameStatus::Playing) && self.board.is_win_condition_met() {
            self.status = GameStatus::Won;
        }
    }

    pub fn status(&self) -> GameStatus {
        self.status
    }

    //TODO: Instead of hardcoding the board size here, take input from UI or some selection
    // to specify board sizing reset(&mut self, x, y, mines)
    pub fn reset(&mut self) {
        self.board = Board::new(10, 10, 10);
        self.status = GameStatus::Created;
    }
}
