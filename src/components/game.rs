use crate::ms_board::{Board, CellContent, CellState, gameplay::RevealResult};
use eframe::egui;

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

    fn can_interact(&self) -> bool {
        !matches!(self.status, GameStatus::Won | GameStatus::Lost)
    }

    // TODO: if GameStatus::Lost isn't handled, clicking mines does nothing
    fn handle_click(&mut self, x: usize, y: usize) {
        if !self.can_interact() {
            return;
        }

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

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if !self.can_interact() {
            return;
        }

        self.board.toggle_flag(self.board.get_index(x, y));
    }

    fn check_win(&mut self) {
        if matches!(self.status, GameStatus::Playing) && self.board.is_win_condition_met() {
            self.status = GameStatus::Won;
        }
    }

    // pub fn toggle_flag(&mut self, x: usize, y: usize) {
    //     self.board.toggle_flag(self.board.get_index(x, y));
    // }

    // pub fn status(&self) -> GameStatus {
    //     self.status
    // }

    //TODO: Instead of hardcoding the board size here, take input from UI or some selection
    // to specify board sizing reset(&mut self, x, y, mines)
    pub fn reset(&mut self, x: usize, y: usize, mines: usize) {
        self.board = Board::new(x, y, mines);
        self.status = GameStatus::Created;
    }

    fn get_cell_display(&self, x: usize, y: usize) -> CellDisplay {
        let (content, state) = self.board.cell_at(x, y);
        match state {
            CellState::Hidden => CellDisplay::Hidden,
            CellState::Flagged => CellDisplay::Flag,
            _ => match content {
                CellContent::Mine => CellDisplay::Mine,
                CellContent::Number(0) => CellDisplay::Empty,
                CellContent::Number(num) => CellDisplay::Number(num),
            },
        }
    }

    fn get_cell_char(&self, x: usize, y: usize) -> char {
        let cell_display = self.get_cell_display(x, y);
        match cell_display {
            CellDisplay::Hidden => '\u{200B}',
            CellDisplay::Flag => '🚩',
            CellDisplay::Mine => '💣',
            CellDisplay::Number(num) => std::char::from_digit(num as u32, 10).unwrap_or('?'),
            CellDisplay::Empty => ' ',
        }
    }

    fn render_status(&self, ui: &mut egui::Ui) {
        match self.status {
            GameStatus::Won => {
                ui.colored_label(egui::Color32::GREEN, " YOU WON ");
            }
            GameStatus::Lost => {
                ui.colored_label(egui::Color32::RED, " YOU LOST ");
            }
            _ => {}
        };
    }

    fn render_board(&mut self, ui: &mut egui::Ui) {
        // TODO: Make the cell size dynamic based on user input field/dropdown.
        // TODO: Create different colors between opened cells and unopened cells.
        // TODO: Make the numbers correlate with dark mode on minesweeper.online.
        // TODO: Change the font size of the numbers to be larger and more readable.

        // Board rendering logic. Each cell is a square of size cell_size, and the board is drawn as a grid of these squares.
        let cell_size = 30.0;
        let width = self.board.get_width();
        let height = self.board.get_height();
        let desired_size = egui::vec2(cell_size * width as f32, cell_size * height as f32);
        // Reserve the exact space needed for the full grid so pointer hits map cleanly to cells.
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        // TODO: If we ever want drag-release behavior across different cells, track press origin and release position separately.
        // For now we intentionally only handle simple click/tap interactions on the cell under the pointer.
        // Convert mouse pointer location into a board cell index for left-click reveal and right-click flagging.
        if let Some(pos) = response.interact_pointer_pos() {
            let local = pos - rect.min;
            let col = (local.x / cell_size).floor() as usize;
            let row = (local.y / cell_size).floor() as usize;
            if response.clicked() {
                self.handle_click(col, row);
                self.check_win();
            }
            if response.secondary_clicked() {
                self.toggle_flag(col, row);
            }
        }

        let painter = ui.painter_at(rect);
        for row in 0..height {
            for col in 0..width {
                let cell_display = self.get_cell_display(col, row);
                let cell_char = self.get_cell_char(col, row);
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(col as f32 * cell_size, row as f32 * cell_size),
                    egui::vec2(cell_size, cell_size),
                );

                self.render_cell(&painter, cell_rect, cell_display, cell_char, ui);
            }
        }
    }

    fn get_number_color(num: u8) -> egui::Color32 {
        match num {
            1 => egui::Color32::from_rgb(124, 199, 255),
            2 => egui::Color32::from_rgb(102, 194, 102),
            3 => egui::Color32::from_rgb(255, 119, 136),
            4 => egui::Color32::from_rgb(238, 136, 255),
            5 => egui::Color32::from_rgb(221, 170, 34),
            6 => egui::Color32::from_rgb(102, 204, 204),
            7 => egui::Color32::from_rgb(153, 153, 153),
            8 => egui::Color32::from_rgb(208, 216, 223),
            _ => egui::Color32::WHITE,
        }
    }

    fn render_cell(
        &self,
        painter: &egui::Painter,
        cell_rect: egui::Rect,
        cell_display: CellDisplay,
        cell_char: char,
        _ui: &egui::Ui,
    ) {
        let bg_color = match cell_display {
            CellDisplay::Hidden | CellDisplay::Flag => egui::Color32::from_rgb(76, 84, 92),
            _ => egui::Color32::from_rgb(56, 64, 72),
        };

        painter.rect_filled(cell_rect, 0.0, bg_color);
        painter.rect_stroke(
            cell_rect,
            0.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgb(30, 38, 46)),
            egui::StrokeKind::Inside,
        );

        if !matches!(cell_display, CellDisplay::Hidden) {
            let text_color = match cell_display {
                CellDisplay::Number(num) => Self::get_number_color(num),
                CellDisplay::Mine => egui::Color32::from_rgb(255, 80, 80),
                CellDisplay::Flag => egui::Color32::from_rgb(255, 120, 0),
                CellDisplay::Empty => egui::Color32::WHITE,
                CellDisplay::Hidden => egui::Color32::WHITE,
            };

            let font_size = (cell_rect.width().min(cell_rect.height()) * 0.64).max(14.0);
            painter.text(
                cell_rect.center(),
                egui::Align2::CENTER_CENTER,
                cell_char.to_string(),
                egui::FontId::new(font_size, egui::FontFamily::Proportional),
                text_color,
            );
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        self.render_status(ui);
        self.render_board(ui);
    }
}
