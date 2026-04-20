use crate::card::{Card, CardType, Direction};
use serde::{Deserialize, Serialize};

pub const BOARD_SIZE: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Owner {
    Blue,
    Red,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Blocked,
    Occupied { card: Card, owner: Owner },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
}

impl Board {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
        }
    }

    pub fn cell(&self, row: usize, col: usize) -> Cell {
        self.cells[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, cell: Cell) {
        self.cells[row][col] = cell;
    }

    /// Place a card at (row, col) and resolve all battles (including counterattacks) and chains.
    pub fn place(
        &mut self,
        row: usize,
        col: usize,
        card: Card,
        owner: Owner,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            matches!(self.cells[row][col], Cell::Empty),
            "cell ({row},{col}) is not empty"
        );
        self.cells[row][col] = Cell::Occupied { card, owner };

        // Seed battle queue: (attacker_pos, defender_pos)
        let mut queue: Vec<((usize, usize), (usize, usize))> = Vec::new();

        for dir in Direction::ALL {
            let (dr, dc) = dir.delta();
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if !(0..BOARD_SIZE as i32).contains(&nr) || !(0..BOARD_SIZE as i32).contains(&nc) {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);

            if let Cell::Occupied {
                card: adj,
                owner: adj_owner,
            } = self.cells[nr][nc]
                && adj_owner != owner
            {
                // Placed card attacks adjacent if it has an arrow toward them.
                if card.has_arrow(dir) {
                    queue.push(((row, col), (nr, nc)));
                }
                // Adjacent card attacks placed card if it has an arrow pointing back.
                if adj.has_arrow(dir.opposite()) {
                    queue.push(((nr, nc), (row, col)));
                }
            }
        }

        // Resolve battles; chains add new entries to the queue.
        let mut flipped = [[false; BOARD_SIZE]; BOARD_SIZE];
        let mut i = 0;
        while i < queue.len() {
            let ((ar, ac), (dr, dc)) = queue[i];
            i += 1;

            let (atk_card, atk_owner) = match self.cells[ar][ac] {
                Cell::Occupied { card, owner } => (card, owner),
                _ => continue,
            };
            let (def_card, def_owner) = match self.cells[dr][dc] {
                Cell::Occupied { card, owner } => (card, owner),
                _ => continue,
            };

            if atk_owner == def_owner {
                continue;
            } // already same team

            let win = atk_card.card_type == CardType::Assault
                || atk_card.win_probability(def_card) >= 0.5;

            if win && !flipped[dr][dc] {
                flipped[dr][dc] = true;
                self.cells[dr][dc] = Cell::Occupied {
                    card: def_card,
                    owner: atk_owner,
                };
                // Chain: flipped card's arrows trigger new battles.
                for dir in Direction::ALL {
                    if !def_card.has_arrow(dir) {
                        continue;
                    }
                    let (ddr, ddc) = dir.delta();
                    let nr = dr as i32 + ddr;
                    let nc = dc as i32 + ddc;
                    if !(0..BOARD_SIZE as i32).contains(&nr)
                        || !(0..BOARD_SIZE as i32).contains(&nc)
                    {
                        continue;
                    }
                    let (nr, nc) = (nr as usize, nc as usize);
                    if let Cell::Occupied {
                        owner: target_owner,
                        ..
                    } = self.cells[nr][nc]
                        && target_owner != atk_owner
                    {
                        queue.push(((dr, dc), (nr, nc)));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn count(&self, owner: Owner) -> usize {
        self.cells
            .iter()
            .flatten()
            .filter(|c| matches!(c, Cell::Occupied { owner: o, .. } if *o == owner))
            .count()
    }

    pub fn empty_cells(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for r in 0..BOARD_SIZE {
            for c in 0..BOARD_SIZE {
                if matches!(self.cells[r][c], Cell::Empty) {
                    out.push((r, c));
                }
            }
        }
        out
    }
}
