use minesweeper_solver::board::Board;

fn main() {
    let mut board = Board::new_empty(10, 10);
    board.generate_mines(0, 0, 10);

    println!("\n--- Board Generation Verification ---\n");
    println!("{}", board);
    
}
