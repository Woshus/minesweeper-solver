use eframe::egui::{self, DragValue};

#[derive(PartialEq, Copy, Clone, Debug)]
enum Difficulty {
    Beginner,
    Intermediate,
    Expert,
    Custom,
}
pub struct DifficultySelector {
    difficulty: Difficulty,
    rows: usize,
    cols: usize,
    mines: usize,
}

impl DifficultySelector {
    pub fn new() -> Self {
        Self {
            difficulty: Difficulty::Beginner,
            rows: 10,
            cols: 10,
            mines: 10,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_value(&mut self.difficulty, Difficulty::Beginner, "Beginner")
                .clicked()
            {
                self.rows = 10;
                self.cols = 10;
                self.mines = 10;
            }
            if ui
                .selectable_value(
                    &mut self.difficulty,
                    Difficulty::Intermediate,
                    "Intermediate",
                )
                .clicked()
            {
                self.rows = 16;
                self.cols = 16;
                self.mines = 40;
            }
            if ui
                .selectable_value(&mut self.difficulty, Difficulty::Expert, "Expert")
                .clicked()
            {
                self.rows = 16;
                self.cols = 30;
                self.mines = 99;
            }
            ui.selectable_value(&mut self.difficulty, Difficulty::Custom, "Custom");
            if self.difficulty == Difficulty::Custom {
                ui.separator();
                ui.label("Rows:");
                ui.add(DragValue::new(&mut self.rows).range(5..=100).speed(0.1));

                ui.label("Cols:");
                ui.add(DragValue::new(&mut self.cols).range(5..=100).speed(0.1));

                ui.label("Mines:");
                ui.add(DragValue::new(&mut self.mines).range(5..=100).speed(0.1));
            }
        });
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn mines(&self) -> usize {
        self.mines
    }
}
