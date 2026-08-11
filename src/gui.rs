use crate::components::{DifficultySelector, MinesweeperGame, prob_solver::ProbabilitySolver};
use eframe::egui;

#[derive(PartialEq, Copy, Clone, Debug)]
enum Tab {
    Game,
    Solver,
}

pub struct MinesweeperSolver {
    active_tab: Tab,
    game: MinesweeperGame,
    solver: ProbabilitySolver,
    difficulty_selector: DifficultySelector,
}

impl MinesweeperSolver {
    fn install_custom_font(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "krona".to_owned(),
            egui::FontData::from_static(include_bytes!("../assets/fonts/KronaOne-Regular.ttf"))
                .into(),
        );

        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "krona".to_owned());

        ctx.set_fonts(fonts);
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::install_custom_font(&cc.egui_ctx);

        Self {
            active_tab: Tab::Game,
            game: MinesweeperGame::new(10, 10, 10),
            solver: ProbabilitySolver::new(10, 10, 10),
            difficulty_selector: DifficultySelector::default(),
        }
    }
}

// TODO: Top Level ui call should handle control flow, functional elements can be placed elsewhere.
impl eframe::App for MinesweeperSolver {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("solver-menu").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                let game_tab = ui.selectable_value(&mut self.active_tab, Tab::Game, "Play Game");
                let solver_tab =
                    ui.selectable_value(&mut self.active_tab, Tab::Solver, "Solver Tool");

                if self.active_tab == Tab::Game {
                    game_tab.highlight();
                } else {
                    solver_tab.highlight();
                }

                ui.separator();
            })
        });

        match self.active_tab {
            Tab::Game => {
                self.difficulty_selector.ui(ui);

                if ui.button("RESTART").clicked() {
                    self.game.reset(
                        self.difficulty_selector.cols(),
                        self.difficulty_selector.rows(),
                        self.difficulty_selector.mines(),
                    );
                }
                self.game.ui(ui);
            }
            Tab::Solver => {
                self.solver.ui(ui);
            }
        }
        // Game Tab

        // TODO : Make it so that the board automatically resets when the difficulty/board size is changed
    }
}
