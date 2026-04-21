use std::time::Duration;

mod board;
mod card;
mod solver;
mod state;
mod tests;
mod tui;

fn main() -> anyhow::Result<()> {
    let time_budget = match std::env::args().nth(1) {
        Some(arg) => {
            let secs: f64 = arg
                .parse()
                .map_err(|_| anyhow::anyhow!("usage: tetra-master-adviser [TIME_BUDGET_SECS]"))?;
            anyhow::ensure!(secs > 0.0, "time budget must be positive");
            Duration::from_secs_f64(secs)
        }
        None => solver::DEFAULT_TIME_BUDGET,
    };
    tui::run(time_budget)
}
