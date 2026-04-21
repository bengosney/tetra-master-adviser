use serde::{Deserialize, Serialize};

/// Arrow directions as bit positions (MSB = North, clockwise).
/// Bit 7=N, 6=NE, 5=E, 4=SE, 3=S, 2=SW, 1=W, 0=NW
pub const ARROW_N: u8 = 0b10000000;
pub const ARROW_NE: u8 = 0b01000000;
pub const ARROW_E: u8 = 0b00100000;
pub const ARROW_SE: u8 = 0b00010000;
pub const ARROW_S: u8 = 0b00001000;
pub const ARROW_SW: u8 = 0b00000100;
pub const ARROW_W: u8 = 0b00000010;
pub const ARROW_NW: u8 = 0b00000001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Direction {
    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::NE,
        Direction::E,
        Direction::SE,
        Direction::S,
        Direction::SW,
        Direction::W,
        Direction::NW,
    ];

    /// Row/col delta when moving in this direction.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::N => (-1, 0),
            Direction::NE => (-1, 1),
            Direction::E => (0, 1),
            Direction::SE => (1, 1),
            Direction::S => (1, 0),
            Direction::SW => (1, -1),
            Direction::W => (0, -1),
            Direction::NW => (-1, -1),
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::N => Direction::S,
            Direction::NE => Direction::SW,
            Direction::E => Direction::W,
            Direction::SE => Direction::NW,
            Direction::S => Direction::N,
            Direction::SW => Direction::NE,
            Direction::W => Direction::E,
            Direction::NW => Direction::SE,
        }
    }

    pub fn arrow_bit(self) -> u8 {
        match self {
            Direction::N => ARROW_N,
            Direction::NE => ARROW_NE,
            Direction::E => ARROW_E,
            Direction::SE => ARROW_SE,
            Direction::S => ARROW_S,
            Direction::SW => ARROW_SW,
            Direction::W => ARROW_W,
            Direction::NW => ARROW_NW,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Physical, // P
    Magic,    // M
    Flexible, // X  — uses whichever defense is lower
    Assault,  // A  — attacks all adjacent cards simultaneously
}

const STAT_RANGE: f32 = 16.0;
const ROLL_MIDPOINT: f32 = 7.5;

/// A Tetra Master card.
/// Stats are 0–15 (the hex digit visible on screen).
/// The true internal value is digit*16 + hidden (0–15); we use digit*16+8 as midpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub attack: u8, // 0–15
    pub card_type: CardType,
    pub phys_def: u8, // 0–15
    pub mag_def: u8,  // 0–15
    pub arrows: u8,   // bitmask
}

impl Card {
    pub fn new(attack: u8, card_type: CardType, phys_def: u8, mag_def: u8, arrows: u8) -> Self {
        Self {
            attack,
            card_type,
            phys_def,
            mag_def,
            arrows,
        }
    }

    /// Expected attack value using the visible digit midpoint.
    pub fn attack_value(self) -> f32 {
        (self.attack as f32) * STAT_RANGE + ROLL_MIDPOINT
    }

    /// Expected defense value against a given attacker type.
    pub fn defense_value(&self, attacker_type: CardType) -> f32 {
        // Pre-calculate the "mean" roll for each stat
        // Formula: (Stat * STAT_RANGE) + ROLL_MIDPOINT (the midpoint of 0..15)
        let p_def = (self.phys_def as f32) * STAT_RANGE + ROLL_MIDPOINT;
        let m_def = (self.mag_def as f32) * STAT_RANGE + ROLL_MIDPOINT;
        let atk_pow = (self.attack as f32) * STAT_RANGE + ROLL_MIDPOINT;

        match attacker_type {
            CardType::Physical => p_def,
            CardType::Magic => m_def,
            CardType::Flexible => p_def.min(m_def),
            CardType::Assault => p_def.min(m_def).min(atk_pow),
        }
    }

    /// Probability that this card wins a battle as the attacker against `defender`.
    /// Models: attacker rolls U[0, atk], defender rolls U[0, def]; attacker wins if atk_roll >= def_roll.
    pub fn win_probability(self, defender: Card) -> f32 {
        let a = self.attack_value();
        let d = defender.defense_value(self.card_type);
        // P(A >= D) where A~U[0,a], D~U[0,d]
        // = integral over continuous uniform approximation
        let a1 = a + 1.0;
        let d1 = d + 1.0;
        // P(A >= D) = 1 - P(A < D) = 1 - (d*(d+1)/2) / (a1*d1)  ... piecewise
        // Simpler closed form for U[0,A] vs U[0,D]:
        if a >= d {
            1.0 - (d1) / (2.0 * a1)
        } else {
            a1 / (2.0 * d1)
        }
    }

    pub fn has_arrow(self, dir: Direction) -> bool {
        self.arrows & dir.arrow_bit() != 0
    }

    /// Parse from "2P34 10110101" format.
    pub fn parse(s: &str) -> anyhow::Result<Card> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        anyhow::ensure!(parts.len() == 2, "expected \"XYYY BBBBBBBB\"");
        let stats = parts[0];
        let bits = parts[1];

        anyhow::ensure!(stats.len() == 4, "stats must be 4 chars e.g. 2P34");
        let atk = u8::from_str_radix(&stats[0..1], 16)?;
        let card_type = match &stats[1..2] {
            "P" | "p" => CardType::Physical,
            "M" | "m" => CardType::Magic,
            "X" | "x" => CardType::Flexible,
            "A" | "a" => CardType::Assault,
            c => anyhow::bail!("unknown card type '{c}'"),
        };
        let pd = u8::from_str_radix(&stats[2..3], 16)?;
        let md = u8::from_str_radix(&stats[3..4], 16)?;

        anyhow::ensure!(
            bits.len() == 8,
            "arrow bitfield must be 8 binary digits e.g. 10110101"
        );
        let arrows = u8::from_str_radix(bits, 2)?;

        Ok(Card::new(atk, card_type, pd, md, arrows))
    }

    pub fn arrow_display(self) -> String {
        let syms = [
            (ARROW_NW, '↖'),
            (ARROW_N, '↑'),
            (ARROW_NE, '↗'),
            (ARROW_W, '←'),
            (ARROW_E, '→'),
            (ARROW_SW, '↙'),
            (ARROW_S, '↓'),
            (ARROW_SE, '↘'),
        ];
        syms.iter()
            .filter(|(bit, _)| self.arrows & bit != 0)
            .map(|(_, ch)| *ch)
            .collect()
    }

    pub fn stat_string(self) -> String {
        let t = match self.card_type {
            CardType::Physical => 'P',
            CardType::Magic => 'M',
            CardType::Flexible => 'X',
            CardType::Assault => 'A',
        };
        format!(
            "{:X}{}{:X}{:X}",
            self.attack, t, self.phys_def, self.mag_def
        )
    }
}
