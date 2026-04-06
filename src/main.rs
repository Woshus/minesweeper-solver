use minesweeper_solver::board::Board;

fn main() {
    let mut board = Board::new(100, 100);
    board.generate_mines(0, 0, 10);

    println!("\n--- Board Generation Verification ---\n");
    println!("{}", board);

    board.click_cell(0);
    println!("\n--- Board Generation Verification ---\n");
    println!("{}", board);
}
