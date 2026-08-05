# Minesweeper Solver

A Rust-based Minesweeper application with an `egui` desktop UI and a probability-based solver engine.

## Overview

This project is a Minesweeper game built with `eframe`/`egui` for the UI and a custom board engine under `src/ms_board`.
The app currently includes:

- A playable Minesweeper board (`src/components/game.rs`)
- A difficulty selector UI component
- A solver tab skeleton for probability-based solving logic (`src/components/prob_solver.rs`)
- Probability computation logic for revealed board frontier cells (`src/ms_board/probabilities.rs`)

## Requirements

- Rust toolchain (stable)
- `cargo`

## Build and Run

From the project root:

```powershell
cargo run
```

This launches the native desktop application.

## Tests

Run the test suite with:

```powershell
cargo test
```

## Project Structure

- `Cargo.toml` — project manifest and dependency list
- `src/main.rs` — application entry point
- `src/gui.rs` — top-level app state and UI tab handling
- `src/components/` — UI components for the game, solver, and difficulty selection
- `src/ms_board/` — Minesweeper board model, gameplay, generation, and probability logic

## Notes

- The solver UI is present, but the solver feature is still under development.
- The board generation and probability computation logic is implemented in `src/ms_board`.

## License

This project is licensed under the MIT License. See `LICENSE.txt` for details.
