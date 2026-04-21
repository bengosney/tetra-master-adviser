#[cfg(test)]
mod card_tests {
    use crate::card::{ARROW_E, ARROW_N, Card, CardType, Direction};

    fn physical(attack: u8, phys_def: u8, arrows: u8) -> Card {
        Card::new(attack, CardType::Physical, phys_def, 0, arrows)
    }

    fn magic(attack: u8, mag_def: u8, arrows: u8) -> Card {
        Card::new(attack, CardType::Magic, 0, mag_def, arrows)
    }

    #[test]
    fn parse_roundtrip() {
        let card = Card::parse("2P34 10110101").unwrap();
        assert_eq!(card.attack, 2);
        assert_eq!(card.card_type, CardType::Physical);
        assert_eq!(card.phys_def, 3);
        assert_eq!(card.mag_def, 4);
        assert_eq!(card.stat_string(), "2P34");
    }

    #[test]
    fn parse_all_types() {
        assert_eq!(
            Card::parse("0P00 00000000").unwrap().card_type,
            CardType::Physical
        );
        assert_eq!(
            Card::parse("0M00 00000000").unwrap().card_type,
            CardType::Magic
        );
        assert_eq!(
            Card::parse("0X00 00000000").unwrap().card_type,
            CardType::Flexible
        );
        assert_eq!(
            Card::parse("0A00 00000000").unwrap().card_type,
            CardType::Assault
        );
    }

    #[test]
    fn parse_max_stats() {
        let card = Card::parse("FPFF 11111111").unwrap();
        assert_eq!(card.attack, 15);
        assert_eq!(card.phys_def, 15);
        assert_eq!(card.mag_def, 15);
        assert_eq!(card.arrows, 0xFF);
    }

    #[test]
    fn parse_invalid_type() {
        assert!(Card::parse("2Z34 10110101").is_err());
    }

    #[test]
    fn parse_wrong_length() {
        assert!(Card::parse("2P3 10110101").is_err());
        assert!(Card::parse("2P34 1011010").is_err());
    }

    #[test]
    fn has_arrow() {
        let card = Card::new(0, CardType::Physical, 0, 0, ARROW_N | ARROW_E);
        assert!(card.has_arrow(Direction::N));
        assert!(card.has_arrow(Direction::E));
        assert!(!card.has_arrow(Direction::S));
        assert!(!card.has_arrow(Direction::W));
    }

    #[test]
    fn win_probability_equal_stats() {
        // Equal attack and defense: roughly 50/50
        let atk = physical(5, 0, 0);
        let def = physical(0, 5, 0);
        let p = atk.win_probability(def);
        assert!((p - 0.5).abs() < 0.01, "expected ~0.5, got {p}");
    }

    #[test]
    fn win_probability_dominant_attacker() {
        let atk = physical(15, 0, 0);
        let def = physical(0, 0, 0);
        let p = atk.win_probability(def);
        assert!(p > 0.9, "dominant attacker should win most of the time");
    }

    #[test]
    fn win_probability_dominant_defender() {
        let atk = physical(0, 0, 0);
        let def = physical(0, 15, 0);
        let p = atk.win_probability(def);
        assert!(p < 0.1, "dominant defender should lose most of the time");
    }

    #[test]
    fn flexible_uses_lower_defense() {
        let atk = Card::new(5, CardType::Flexible, 0, 0, 0);
        let def_high_phys = Card::new(0, CardType::Physical, 15, 1, 0);
        let def_high_mag = Card::new(0, CardType::Physical, 1, 15, 0);
        // Flexible attacker picks the lower of phys/mag defense
        let p_phys = atk.win_probability(def_high_phys); // low mag_def used
        let p_mag = atk.win_probability(def_high_mag); // low phys_def used
        assert!(p_phys > 0.5, "flexible should beat low mag_def ({p_phys})");
        assert!(p_mag > 0.5, "flexible should beat low phys_def ({p_mag})");
    }

    #[test]
    fn magic_uses_mag_def() {
        let atk = magic(8, 0, 0);
        let high_phys_low_mag = Card::new(0, CardType::Physical, 15, 0, 0);
        let p = atk.win_probability(high_phys_low_mag);
        assert!(p > 0.5, "magic attacker should ignore high phys_def");
    }
}

#[cfg(test)]
mod board_tests {
    use crate::board::{Board, Cell, Owner};
    use crate::card::{ARROW_E, ARROW_S, ARROW_W, Card, CardType, Direction};

    fn card_facing(dirs: &[Direction]) -> Card {
        let arrows = dirs.iter().fold(0u8, |acc, d| acc | d.arrow_bit());
        Card::new(15, CardType::Physical, 0, 0, arrows)
    }

    fn weak_card() -> Card {
        Card::new(0, CardType::Physical, 0, 0, 0)
    }

    fn strong_card(arrows: u8) -> Card {
        Card::new(15, CardType::Physical, 15, 15, arrows)
    }

    #[test]
    fn place_on_empty_cell() {
        let mut board = Board::new();
        let card = weak_card();
        assert!(board.place(0, 0, card, Owner::Blue).is_ok());
        assert!(matches!(
            board.cell(0, 0),
            Cell::Occupied {
                owner: Owner::Blue,
                ..
            }
        ));
    }

    #[test]
    fn place_on_occupied_fails() {
        let mut board = Board::new();
        let card = weak_card();
        board.place(0, 0, card, Owner::Blue).unwrap();
        assert!(board.place(0, 0, card, Owner::Red).is_err());
    }

    #[test]
    fn no_battle_without_arrow() {
        let mut board = Board::new();
        // Red card at (0,1) with no arrows
        board.set(
            0,
            1,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Red,
            },
        );
        // Blue card placed at (0,0) with no arrows — no battle should occur
        board.place(0, 0, weak_card(), Owner::Blue).unwrap();
        assert!(matches!(
            board.cell(0, 1),
            Cell::Occupied {
                owner: Owner::Red,
                ..
            }
        ));
    }

    #[test]
    fn attacker_flips_weaker_adjacent() {
        let mut board = Board::new();
        // Red weak card at (0,1)
        board.set(
            0,
            1,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Red,
            },
        );
        // Blue strong card placed at (0,0) facing East — battles Red
        board
            .place(0, 0, strong_card(ARROW_E), Owner::Blue)
            .unwrap();
        assert!(
            matches!(
                board.cell(0, 1),
                Cell::Occupied {
                    owner: Owner::Blue,
                    ..
                }
            ),
            "red card should have been flipped to blue"
        );
    }

    #[test]
    fn defender_counterattacks() {
        let mut board = Board::new();
        // Red strong card at (0,1) facing West (back toward blue)
        board.set(
            0,
            1,
            Cell::Occupied {
                card: strong_card(ARROW_W),
                owner: Owner::Red,
            },
        );
        // Blue weak card placed at (0,0) with East arrow — attacks Red, Red counters
        board.place(0, 0, weak_card(), Owner::Red).unwrap(); // dummy, just setup
        let mut board2 = Board::new();
        board2.set(
            0,
            1,
            Cell::Occupied {
                card: strong_card(ARROW_W),
                owner: Owner::Red,
            },
        );
        board2
            .place(
                0,
                0,
                Card::new(0, CardType::Physical, 0, 0, ARROW_E),
                Owner::Blue,
            )
            .unwrap();
        // Blue attacked Red (will lose), Red countered — blue card should be flipped
        assert!(
            matches!(
                board2.cell(0, 0),
                Cell::Occupied {
                    owner: Owner::Red,
                    ..
                }
            ),
            "blue weak card should have been flipped by red counterattack"
        );
    }

    #[test]
    fn chain_flip_propagates() {
        let mut board = Board::new();
        // Layout: Blue places at (0,0) facing S
        //         Red weak at (1,0) facing S
        //         Red weak at (2,0) facing nowhere
        board.set(
            1,
            0,
            Cell::Occupied {
                card: card_facing(&[Direction::S]),
                owner: Owner::Red,
            },
        );
        board.set(
            2,
            0,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Red,
            },
        );
        // Blue strong card at (0,0) facing S — flips (1,0), which chains S to flip (2,0)
        board
            .place(0, 0, strong_card(ARROW_S), Owner::Blue)
            .unwrap();
        assert!(
            matches!(
                board.cell(1, 0),
                Cell::Occupied {
                    owner: Owner::Blue,
                    ..
                }
            ),
            "(1,0) should be flipped"
        );
        assert!(
            matches!(
                board.cell(2, 0),
                Cell::Occupied {
                    owner: Owner::Blue,
                    ..
                }
            ),
            "(2,0) should be chain-flipped"
        );
    }

    #[test]
    fn count_owners() {
        let mut board = Board::new();
        board.set(
            0,
            0,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Blue,
            },
        );
        board.set(
            0,
            1,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Blue,
            },
        );
        board.set(
            0,
            2,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Red,
            },
        );
        assert_eq!(board.count(Owner::Blue), 2);
        assert_eq!(board.count(Owner::Red), 1);
    }

    #[test]
    fn empty_cells() {
        let mut board = Board::new();
        board.set(
            0,
            0,
            Cell::Occupied {
                card: weak_card(),
                owner: Owner::Blue,
            },
        );
        board.set(0, 1, Cell::Blocked);
        let empty = board.empty_cells();
        assert_eq!(empty.len(), 14);
        assert!(!empty.contains(&(0, 0)));
        assert!(!empty.contains(&(0, 1)));
    }
}

#[cfg(test)]
mod solver_tests {
    use crate::board::{Board, Cell, Owner};
    use crate::card::{ARROW_E, Card, CardType};
    use std::time::Duration;

    use crate::solver::{DEFAULT_TIME_BUDGET, best_move};

    fn strong_east() -> Card {
        Card::new(15, CardType::Physical, 15, 15, ARROW_E)
    }

    #[test]
    fn no_move_with_empty_hand() {
        let board = Board::new();
        assert!(best_move(&board, &[], DEFAULT_TIME_BUDGET).is_none());
    }

    #[test]
    fn finds_a_move_on_empty_board() {
        let board = Board::new();
        let hand = vec![strong_east()];
        let m = best_move(&board, &hand, DEFAULT_TIME_BUDGET).unwrap();
        assert!(m.row < 4 && m.col < 4);
        assert_eq!(m.card_index, 0);
    }

    #[test]
    fn prefers_capturing_red_card() {
        let mut board = Board::new();
        // Red weak card sitting at (0,1) — placing strong E-arrow card at (0,0) should capture it
        board.set(
            0,
            1,
            Cell::Occupied {
                card: Card::new(0, CardType::Physical, 0, 0, 0),
                owner: Owner::Red,
            },
        );
        let hand = vec![strong_east()];
        let m = best_move(&board, &hand, DEFAULT_TIME_BUDGET).unwrap();
        assert_eq!(
            (m.row, m.col),
            (0, 0),
            "solver should place at (0,0) to capture the red card"
        );
    }

    #[test]
    fn min_depth_completes_with_zero_budget() {
        // Even with near-zero time budget, solver must return a result
        // because MIN_DEPTH runs without time checks.
        let board = Board::new();
        let hand = vec![strong_east()];
        let m = best_move(&board, &hand, Duration::from_nanos(1)).unwrap();
        assert!(m.row < 4 && m.col < 4);
    }

    #[test]
    fn longer_budget_still_returns_valid_move() {
        let board = Board::new();
        let hand = vec![strong_east()];
        let m = best_move(&board, &hand, Duration::from_secs(1)).unwrap();
        assert!(m.row < 4 && m.col < 4);
        assert_eq!(m.card_index, 0);
    }

    #[test]
    fn result_consistent_across_budgets() {
        // With a nearly-filled board, search is small enough to complete fully
        // regardless of budget — results should match.
        let mut board = Board::new();
        let filler = Card::new(1, CardType::Physical, 1, 1, 0);
        // Fill all but two cells
        for r in 0..4 {
            for c in 0..4 {
                if (r, c) != (3, 2) && (r, c) != (3, 3) {
                    let owner = if (r + c) % 2 == 0 {
                        Owner::Blue
                    } else {
                        Owner::Red
                    };
                    board.set(r, c, Cell::Occupied { card: filler, owner });
                }
            }
        }
        let hand = vec![strong_east()];
        let short = best_move(&board, &hand, Duration::from_nanos(1)).unwrap();
        let long = best_move(&board, &hand, Duration::from_secs(5)).unwrap();
        assert_eq!(
            (short.row, short.col),
            (long.row, long.col),
            "small board should produce same result regardless of budget"
        );
    }

    #[test]
    fn multiple_cards_in_hand() {
        let board = Board::new();
        let hand = vec![
            Card::new(1, CardType::Physical, 1, 1, ARROW_E),
            Card::new(15, CardType::Physical, 15, 15, ARROW_E),
            Card::new(5, CardType::Magic, 5, 5, ARROW_E),
        ];
        let m = best_move(&board, &hand, DEFAULT_TIME_BUDGET).unwrap();
        assert!(m.card_index < 3);
        assert!(m.row < 4 && m.col < 4);
    }
}
