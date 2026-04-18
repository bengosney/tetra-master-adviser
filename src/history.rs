use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;

use crate::board::{Board, Cell, Owner};
use crate::card::Card;

fn db_path() -> PathBuf {
    let mut path = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("."))
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    path.push("tetramaster_history.db");
    path
}

pub fn open() -> Result<Connection> {
    let conn = Connection::open(db_path())?;
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS games (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            started   TEXT NOT NULL DEFAULT (datetime('now')),
            result    TEXT          -- 'win', 'loss', 'draw', NULL if in progress
        );

        CREATE TABLE IF NOT EXISTS moves (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id     INTEGER NOT NULL REFERENCES games(id),
            move_num    INTEGER NOT NULL,
            -- card played
            atk         INTEGER NOT NULL,
            card_type   TEXT    NOT NULL,
            phys_def    INTEGER NOT NULL,
            mag_def     INTEGER NOT NULL,
            arrows      INTEGER NOT NULL,
            -- placement
            row         INTEGER NOT NULL,
            col         INTEGER NOT NULL,
            solver_score INTEGER NOT NULL,
            board_before TEXT   NOT NULL  -- JSON snapshot
        );

        CREATE TABLE IF NOT EXISTS cards_seen (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            atk         INTEGER NOT NULL,
            card_type   TEXT    NOT NULL,
            phys_def    INTEGER NOT NULL,
            mag_def     INTEGER NOT NULL,
            arrows      INTEGER NOT NULL,
            owner       TEXT    NOT NULL,  -- 'Blue' or 'Red'
            first_seen  TEXT    NOT NULL DEFAULT (datetime('now')),
            times_seen  INTEGER NOT NULL DEFAULT 1,
            UNIQUE(atk, card_type, phys_def, mag_def, arrows, owner)
        );
    ",
    )?;
    Ok(conn)
}

pub fn new_game(conn: &Connection) -> Result<i64> {
    conn.execute("INSERT INTO games DEFAULT VALUES", [])?;
    Ok(conn.last_insert_rowid())
}

pub struct MoveRecord {
    pub game_id: i64,
    pub move_num: i32,
    pub card: Card,
    pub row: usize,
    pub col: usize,
    pub score: i32,
    pub board: Board,
}

pub fn record_move(conn: &Connection, m: &MoveRecord) -> Result<()> {
    let board = &m.board;
    let card = m.card;
    let (game_id, move_num, row, col, score) = (m.game_id, m.move_num, m.row, m.col, m.score);
    let board_json = serde_json::to_string(board)?;
    let card_type = format!("{:?}", card.card_type);
    conn.execute(
        "INSERT INTO moves (game_id, move_num, atk, card_type, phys_def, mag_def, arrows, row, col, solver_score, board_before)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![game_id, move_num, card.attack, card_type, card.phys_def, card.mag_def, card.arrows, row, col, score, board_json],
    )?;
    Ok(())
}

pub fn record_result(conn: &Connection, game_id: i64, result: &str) -> Result<()> {
    conn.execute(
        "UPDATE games SET result = ?1 WHERE id = ?2",
        params![result, game_id],
    )?;
    Ok(())
}

/// Record every card currently visible on the board.
pub fn record_cards_seen(conn: &Connection, board: &Board) -> Result<()> {
    for row in &board.cells {
        for cell in row {
            if let Cell::Occupied { card, owner } = cell {
                let card_type = format!("{:?}", card.card_type);
                let owner_str = if *owner == Owner::Blue { "Blue" } else { "Red" };
                conn.execute(
                    "INSERT INTO cards_seen (atk, card_type, phys_def, mag_def, arrows, owner)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(atk, card_type, phys_def, mag_def, arrows, owner)
                     DO UPDATE SET times_seen = times_seen + 1",
                    params![
                        card.attack,
                        card_type,
                        card.phys_def,
                        card.mag_def,
                        card.arrows,
                        owner_str
                    ],
                )?;
            }
        }
    }
    Ok(())
}
