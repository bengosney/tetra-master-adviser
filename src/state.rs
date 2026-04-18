use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::board::Board;
use crate::card::Card;

#[derive(Serialize, Deserialize)]
pub struct SaveState {
    pub board: Board,
    pub hand: Vec<Card>,
    pub cursor: (usize, usize),
}

fn save_path() -> PathBuf {
    // Save next to the binary, or fall back to current directory.
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    path.push("tetramaster_save.json");
    path
}

pub fn save(state: &SaveState) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(save_path(), json)?;
    Ok(())
}

pub fn load() -> Option<SaveState> {
    let data = std::fs::read_to_string(save_path()).ok()?;
    serde_json::from_str(&data).ok()
}
