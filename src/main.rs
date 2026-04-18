mod board;
mod card;
mod history;
mod solver;
mod state;
mod tui;

fn main() -> anyhow::Result<()> {
    tui::run()
}
