use std::time::{Duration, Instant};

use crate::board::{BOARD_SIZE, Board, Owner};
use crate::card::{Card, CardType};

#[derive(Debug, Clone)]
pub struct Move {
    pub card_index: usize,
    pub row: usize,
    pub col: usize,
    pub score: i32,
}

// Representative "average" opponent card used to model Red's responses.
// All arrows, moderate stats — pessimistic but reasonable.
const RED_PROXY: Card = Card {
    attack: 2,
    card_type: CardType::Physical,
    phys_def: 2,
    mag_def: 2,
    arrows: 0xFF,
};

const MIN_DEPTH: u32 = 3;
const MAX_DEPTH: u32 = (BOARD_SIZE * BOARD_SIZE) as u32;
pub const DEFAULT_TIME_BUDGET: Duration = Duration::from_secs(2);

/// Find best move using iterative deepening with a time budget.
/// Starts at MIN_DEPTH, increases until time runs out or tree fully searched.
pub fn best_move(board: &Board, hand: &[Card], time_budget: Duration) -> Option<Move> {
    if hand.is_empty() {
        return None;
    }

    let deadline = Instant::now() + time_budget;
    let mut best: Option<Move> = None;

    for depth in MIN_DEPTH..=MAX_DEPTH {
        // MIN_DEPTH always runs fully, ignoring time budget.
        let effective_deadline = if depth > MIN_DEPTH {
            Some(deadline)
        } else {
            None
        };
        match search_at_depth(board, hand, depth, effective_deadline) {
            Some(m) => best = Some(m),
            None => break, // timed out — use previous depth result
        }

        if depth > MIN_DEPTH && Instant::now() >= deadline {
            break;
        }
    }

    best
}

/// Run full search at a given depth. Returns None if deadline exceeded.
/// Pass `None` for deadline to run without time checks.
fn search_at_depth(
    board: &Board,
    hand: &[Card],
    depth: u32,
    deadline: Option<Instant>,
) -> Option<Move> {
    let empty = board.empty_cells();
    let mut best: Option<Move> = None;
    let mut best_score = i32::MIN;
    let mut working_hand = hand.to_vec();
    let last = working_hand.len() - 1;

    for ci in 0..working_hand.len() {
        let card = working_hand[ci];
        for &(row, col) in &empty {
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return None;
            }
            let mut sim = board.clone();
            if sim.place(row, col, card, Owner::Blue).is_err() {
                continue;
            }
            working_hand.swap(ci, last);
            let score = minimax(
                &sim,
                &mut working_hand[..last],
                depth - 1,
                false,
                i32::MIN,
                i32::MAX,
                deadline,
            );
            working_hand.swap(ci, last);
            if score > best_score {
                best_score = score;
                best = Some(Move {
                    card_index: ci,
                    row,
                    col,
                    score,
                });
            }
        }
    }

    Some(best?)
}

fn evaluate(board: &Board) -> i32 {
    board.count(Owner::Blue) as i32 - board.count(Owner::Red) as i32
}

fn minimax(
    board: &Board,
    blue_hand: &mut [Card],
    depth: u32,
    blue_turn: bool,
    mut alpha: i32,
    mut beta: i32,
    deadline: Option<Instant>,
) -> i32 {
    let empty = board.empty_cells();
    if depth == 0 || empty.is_empty() {
        return evaluate(board);
    }

    if blue_turn {
        if blue_hand.is_empty() {
            return minimax(board, blue_hand, depth - 1, false, alpha, beta, deadline);
        }
        let last = blue_hand.len() - 1;
        let mut best = i32::MIN;
        'outer: for ci in 0..=last {
            let card = blue_hand[ci];
            for &(row, col) in &empty {
                if deadline.is_some_and(|d| Instant::now() >= d) {
                    return evaluate(board);
                }
                let mut sim = board.clone();
                if sim.place(row, col, card, Owner::Blue).is_err() {
                    continue;
                }
                blue_hand.swap(ci, last);
                let score = minimax(
                    &sim,
                    &mut blue_hand[..last],
                    depth - 1,
                    false,
                    alpha,
                    beta,
                    deadline,
                );
                blue_hand.swap(ci, last);
                if score > best {
                    best = score;
                }
                if score > alpha {
                    alpha = score;
                }
                if beta <= alpha {
                    break 'outer;
                }
            }
        }
        best
    } else {
        let mut best = i32::MAX;
        'outer: for &(row, col) in &empty {
            if deadline.is_some_and(|d| Instant::now() >= d) {
                return evaluate(board);
            }
            let mut sim = board.clone();
            if sim.place(row, col, RED_PROXY, Owner::Red).is_err() {
                continue;
            }
            let score = minimax(&sim, blue_hand, depth - 1, true, alpha, beta, deadline);
            if score < best {
                best = score;
            }
            if score < beta {
                beta = score;
            }
            if beta <= alpha {
                break 'outer;
            }
        }
        best
    }
}
