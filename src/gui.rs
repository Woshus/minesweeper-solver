use crate::components::{DifficultySelector, MinesweeperGame, game::CellDisplay, game::GameStatus};
use eframe::egui;
use std::char;
pub struct MinesweeperSolver {
    game: MinesweeperGame,
    difficulty_selector: DifficultySelector,
}

impl MinesweeperSolver {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            game: MinesweeperGame::new(10, 10, 10),
            difficulty_selector: DifficultySelector::new(),
        }
    }
}

// TODO: Top Level ui call should handle control flow, functional elements can be placed elsewhere.
impl eframe::App for MinesweeperSolver {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        
        // Game Tab 
        
         // TODO : Make it so that the board automatically resets when the difficulty/board size is changed
        let _difficulty_info = self.difficulty_selector.ui(ui);

        if ui.button("RESTART").clicked() {
            self.game.reset(
                self.difficulty_selector.cols(),
                self.difficulty_selector.rows(),
                self.difficulty_selector.mines(),
            );
        }

        self.game.ui(ui);
    }
}
