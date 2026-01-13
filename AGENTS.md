# Repository Guidelines

## Project Structure & Module Organization
- `src/main.rs` is the entry point; it wires the CPU to the UI and loads a ROM path from CLI args.
- Core emulator components live in `src/`: `cpu.rs`, `ppu.rs`, `bus.rs`, `cartridge.rs`, and `opcodes.rs`.
- UI and rendering code live in `src/ui.rs` and `src/renderer.rs` (egui/eframe).
- Sample asset: `src/nestest.nes` is a known ROM useful for smoke checks.

## Build, Test, and Development Commands
- `cargo build` compiles the emulator.
- `cargo run -- path/to/game.nes` runs the UI and loads the ROM at startup.
- `cargo test` runs the test suite (currently minimal/none; add tests as features land).
- `cargo fmt` and `cargo clippy` keep formatting and linting consistent.

## Coding Style & Naming Conventions
- Rust 2021 edition; use default rustfmt rules (no local config present).
- Keep files/modules in `snake_case` (e.g., `cartridge.rs`), types in `CamelCase`, and constants in `SCREAMING_SNAKE_CASE`.
- Prefer clear separation between emulation core (CPU/PPU/bus) and UI/rendering logic.

## Testing Guidelines
- Add unit tests near the code using `#[cfg(test)]` in module files, or integration tests under `tests/`.
- Name tests descriptively, e.g., `#[test] fn cpu_brk_sets_flags()` to document behavior.
- Use `cargo test` for all test runs; consider using `src/nestest.nes` in focused integration tests.

## Commit & Pull Request Guidelines
- Commit messages are short and capitalized; recent history uses past tense like `Added PPU`.
- PRs should include a concise summary, testing notes (`cargo test`, manual ROM run), and screenshots for UI changes.
- Link related issues or ROMs used for verification when applicable.
