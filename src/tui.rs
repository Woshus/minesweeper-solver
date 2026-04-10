use crate::board::{Board, RevealResult};
use std::io::{self};

// TODO: Reveal all cells on lost game
// TODO: Write tests
pub enum GameStatus {
    Created,
    Playing,
    Won,
    Lost,
}

pub struct TuiMinesweeperGame {
    board: Board,
    status: GameStatus,
}

impl TuiMinesweeperGame {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Self {
            board: Board::new(width, height, mines),
            status: GameStatus::Created,
        }
    }

    pub fn run(&mut self) {
        println!("--- Minesweeper TUI ---");
        println!("Commands c x y (click), f x y (flag), q (quit)");

        while !matches!(self.status, GameStatus::Won | GameStatus::Lost) {
            println!("Board: \n{}", self.board);
            println!("Enter move: ");

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim().to_lowercase();

            if input == "q" {
                break;
            }

            if let Some((action, x, y)) = self.parse_input(&input) {
                let idx = self.board.get_index(x, y);
                self.handle_action(action, idx);
                self.check_win();
            } else {
                println!("Invalid input. Format: c 0 0");
            }
        }

        println!("{}", self.board);
        match self.status {
            GameStatus::Won => println!("You Win!"),
            GameStatus::Lost => println!("You Lost!"),
            _ => println!("Game Exited."),
        }
    }

    fn handle_action(&mut self, action: char, idx: usize) {
        match action {
            'c' => self.handle_click(idx),
            'f' => self.board.toggle_flag(idx),
            _ => println!("Unknown action: {}", action),
        }
    }

    fn handle_click(&mut self, idx: usize) {
        // generate mines on first click
        if matches!(self.status, GameStatus::Created) {
            self.board.generate_mines(idx);
            self.status = GameStatus::Playing
        }

        if self.board.click_cell(idx) == RevealResult::HitMine {
            self.status = GameStatus::Lost;
        }
    }

    fn parse_input(&self, input: &str) -> Option<(char, usize, usize)> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 3 {
            return None;
        }

        let action = parts[0].chars().next()?;
        let x = parts[1].parse::<usize>().ok()?;
        let y = parts[2].parse::<usize>().ok()?;

        Some((action, x, y))
    }

    fn check_win(&mut self) {
        if matches!(self.status, GameStatus::Playing) && self.board.is_win_condition_met() {
            self.status = GameStatus::Won;
        }
    }
}
