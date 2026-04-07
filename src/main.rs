mod tui;

fn main() {
    let mut game = tui::TuiMinesweeperGame::new(10, 10, 10);
    game.run();
}
