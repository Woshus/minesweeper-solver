use eframe::egui;
use minesweeper_solver::gui::MinesweeperSolver;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Minesweeper Solver",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(MinesweeperSolver::new(cc)))
        }),
    )
}
