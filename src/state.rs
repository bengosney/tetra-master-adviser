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
    std::env::temp_dir().join("tetramaster_save.json")
}

pub fn save(state: &SaveState) -> Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(save_path(), json)?;
    Ok(())
}

pub fn load() -> Result<Option<SaveState>> {
    let path = save_path();
    if !path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&path)?;
    let state = serde_json::from_str(&data)?;
    Ok(Some(state))
}
