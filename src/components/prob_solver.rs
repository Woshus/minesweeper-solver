// /*  TODO: This file will contain a UI component that is similar to game.rs.
//     Additional Feaures Include:
//     - Probability Calculation of Hidden Cells based on what is currently revealed
//     - Board Editing
//     - Board is still playable

//     Long-Term Improvements:
//     - Probabilities of both safety and progress (a click leads to further logic)

// */
use eframe::egui;

use crate::ms_board::{Board, CellContent, CellState};

pub enum SolverStatus {
    Created,
    Editing,
    Hidden,
    Shown,
    Finished,
}

pub enum CellDisplay {
    Number(u8),
    Mine,
    Flag,
    Hidden,
    Empty,
    Probability(f32),
}
pub struct ProbabilitySolver {
    board: Board,
    #[allow(dead_code)]
    status: SolverStatus,
}

impl ProbabilitySolver {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self {
            board: Board::new(width, height, mines),
            status: SolverStatus::Created,
        }
    }

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

    fn char_display_iter(&self) -> impl Iterator<Item = char> {
        self.get_display_iter()
            .map(|cell_display| match cell_display {
                CellDisplay::Empty => ' ',
                CellDisplay::Flag => '🚩',
                CellDisplay::Mine => '💣',
                CellDisplay::Number(num) => std::char::from_digit(num as u32, 10).unwrap_or('?'),
                CellDisplay::Hidden => '■',
                _ => ' ',
            })
    }

    fn handle_click(&mut self, _x: usize, _y: usize) {
        // generate mines on first click
        // if matches!(self.status, GameStatus::Created) {
        //     self.board.generate_mines(self.board.get_index(x, y));
        //     self.status = GameStatus::Playing
        // }

        // if self.board.click_cell(self.board.get_index(x, y)) == RevealResult::HitMine {
        //     self.status = GameStatus::Lost;
        //     self.board.reveal_all_mines();
        // }
    }

    fn check_win(&mut self) {
        // if matches!(self.status, GameStatus::Playing) && self.board.is_win_condition_met() {
        //     self.status = GameStatus::Won;
        // }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // let mut can_play = false;
        // match self.status {
        //     GameStatus::Won => {
        //         ui.colored_label(egui::Color32::GREEN, " YOU WON ");
        //     }
        //     GameStatus::Lost => {
        //         ui.colored_label(egui::Color32::RED, " YOU LOST ");
        //     }
        //     _ => {
        //         can_play = true;
        //     }
        // };

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

                            ui.add_enabled_ui(true, |ui| {
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
