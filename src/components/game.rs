use eframe::egui;

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
    
    // TODO: if GameStatus::Lost isn't handled, clicking mines does nothing
    fn handle_click(&mut self, x: usize, y: usize) {
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

    // pub fn toggle_flag(&mut self, x: usize, y: usize) {
    //     self.board.toggle_flag(self.board.get_index(x, y));
    // }

    fn check_win(&mut self) {
        if matches!(self.status, GameStatus::Playing) && self.board.is_win_condition_met() {
            self.status = GameStatus::Won;
        }
    }

    // pub fn status(&self) -> GameStatus {
    //     self.status
    // }

    //TODO: Instead of hardcoding the board size here, take input from UI or some selection
    // to specify board sizing reset(&mut self, x, y, mines)
    pub fn reset(&mut self, x: usize, y: usize, mines: usize) {
        self.board = Board::new(x, y, mines);
        self.status = GameStatus::Created;
    }


    // Returns an iterator of what should be displayed on the board at any given time.
    // Up to the UI for what exactly to display based on what should be displayed.
    fn get_display_iter(&self) -> impl Iterator<Item = CellDisplay> {
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

    // Transforms items from get_display_iter() to usable char values to display
    fn char_display_iter(&self) -> impl Iterator<Item = char> {
        self.get_display_iter()
            .map(|cell_display| match cell_display {
                CellDisplay::Empty => ' ',
                CellDisplay::Flag => '🚩',
                CellDisplay::Mine => '💣',
                CellDisplay::Number(num) => std::char::from_digit(num as u32, 10).unwrap_or('?'),
                CellDisplay::Hidden => '■',
            })
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {

        let mut can_play = false;
        match self.status {
            GameStatus::Won => {
                ui.colored_label(egui::Color32::GREEN, " YOU WON ");
            }
            GameStatus::Lost => {
                ui.colored_label(egui::Color32::RED, " YOU LOST ");
            }
            _ => {
                can_play = true;
            }
        };


        ui.vertical_centered(|ui| {
            ui.heading("Minesweeper Game");
            ui.add_space(10.0);

            let display_chars: Vec<char> = self.char_display_iter().collect();
            let mut dchar_idx = 0;

            egui::Grid::new("minesweeper_grid")
                .spacing([2.0, 2.0])
                .show(ui, |ui| {

                    let width = self.board.get_width();
                    let height = self.board.get_height();

                    for row in 0..height {
                        for col in 0..width {
                            let cell_char = display_chars[dchar_idx];
                            dchar_idx += 1;

                            ui.add_enabled_ui(can_play, |ui| {
                                let button = egui::Button::new(cell_char.to_string())
                                    .min_size(egui::vec2(30.0, 30.0));

                                let response = ui.add(button);
                                if response.clicked() {
                                    self.handle_click(col, row);
                                    self.check_win();
                                }

                                if response.secondary_clicked() {
                                    self.board.toggle_flag(self.board.get_index(col, row));
                                }
                            });
                        }
                        ui.end_row();
                    }
                })
        });
    }
}
