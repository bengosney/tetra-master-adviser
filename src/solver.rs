use crate::board::{Board, Owner};
use crate::card::Card;

#[derive(Debug, Clone)]
pub struct Move {
    pub card_index: usize,
    pub row: usize,
    pub col: usize,
    pub score: i32, // net cards gained for Blue after the move
}

/// Find the best move for Blue given the current board and Blue's hand.
/// Score = Blue card count after placement - Blue card count before placement.
pub fn best_move(board: &Board, hand: &[Card]) -> Option<Move> {
    let before = board.count(Owner::Blue) as i32;
    let empty = board.empty_cells();

    let mut best: Option<Move> = None;

    for (ci, &card) in hand.iter().enumerate() {
        for &(row, col) in &empty {
            let mut sim = board.clone();
            if sim.place(row, col, card, Owner::Blue).is_err() {
                continue;
            }
            let score = sim.count(Owner::Blue) as i32 - before;
            let better = match &best {
                None => true,
                Some(b) => score > b.score,
            };
            if better {
                best = Some(Move {
                    card_index: ci,
                    row,
                    col,
                    score,
                });
            }
        }
    }

    best
}
