use crate::game::{self, CellDisplay, GameStatus};
use eframe::egui;
use std::char;
pub struct MinesweeperSolver {
    game: game::MinesweeperGame,
}

// impl Default for MinesweeperSolver {
//     fn default() -> Self {
//         Self {
//             name: "Arthur".to_owned(),
//             age: 42,
//         }
//     }
// }

impl MinesweeperSolver {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game: game::MinesweeperGame::new(10, 10, 10),
        }
    }

    pub fn char_display_iter(&self) -> impl Iterator<Item = char> {
        self.game
            .get_display_iter()
            .map(|cell_display| match cell_display {
                CellDisplay::Empty => ' ',
                CellDisplay::Flag => '🚩',
                CellDisplay::Mine => '💣',
                CellDisplay::Number(num) => std::char::from_digit(num as u32, 10).unwrap_or('?'),
                CellDisplay::Hidden => '■',
            })
    }
}

impl eframe::App for MinesweeperSolver {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut can_play = false;
        match self.game.status() {
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

        if ui.button("RESTART").clicked() {
            self.game.reset();
        }

        ui.vertical_centered(|ui| {
            ui.heading("Minesweeper Game");
            ui.add_space(10.0);

            let display_chars: Vec<char> = self.char_display_iter().collect();
            let mut dchar_idx = 0;

            egui::Grid::new("minesweeper_grid")
                .spacing([2.0, 2.0])
                .show(ui, |ui| {
                    let (width, height) = self.game.get_dimensions();

                    for row in 0..height {
                        for col in 0..width {
                            let cell_char = display_chars[dchar_idx];
                            dchar_idx += 1;

                            ui.add_enabled_ui(can_play, |ui| {
                                let button = egui::Button::new(cell_char.to_string())
                                    .min_size(egui::vec2(30.0, 30.0));

                                let response = ui.add(button);
                                if response.clicked() {
                                    self.game.handle_click(col, row);
                                    self.game.check_win();
                                }

                                if response.secondary_clicked() {
                                    self.game.toggle_flag(col, row);
                                }
                            });
                        }
                        ui.end_row();
                    }
                })
        });
    }
}
