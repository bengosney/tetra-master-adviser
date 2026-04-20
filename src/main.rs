mod board;
mod card;
mod solver;
mod state;
mod tests;
mod tui;

fn main() -> anyhow::Result<()> {
    tui::run()
}
