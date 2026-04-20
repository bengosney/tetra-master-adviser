# Tetra Master Adviser

A terminal UI adviser for the Tetra Master card game in Final Fantasy IX. Mirror the current board state, add your hand, and the adviser will recommend the best move using minimax with alpha-beta pruning.

> **Warning:** Tetra Master contains hidden stats and random number rolls - this adviser works with expected values and cannot guarantee a win. It will give you the best odds, not a perfect result.

## Installation

Download the latest release for your platform from the [releases page](https://github.com/bengosney/tetra-master-adviser/releases).

### Building from source

Requires Rust 1.85 or later.

```
cargo build --release
```

## Usage

Run the binary from your terminal:

```
./tetra-master-adviser
```

State is saved automatically to your system's temp directory and restored on next launch.

### Controls

| Key       | Action                       |
| --------- | ---------------------------- |
| `↑↓←→`    | Move cursor                  |
| `i`       | Add card to hand             |
| `[` / `]` | Select hand card             |
| `p`       | Place selected card on board |
| `e`       | Place opponent card on board |
| `f`       | Flip card colour             |
| `b`       | Block cell                   |
| `Space`   | Solve - find best move       |
| `r`       | Reset board                  |
| `q`       | Quit and save                |

### Entering cards

Cards are entered in the format `XTYY BBBBBBBB` where:

- `X` - attack stat (hex digit, 0–F)
- `T` - card type: `P` Physical, `M` Magic, `X` Flexible, `A` Assault
- `Y` `Y` - physical and magic defence stats (hex digits, 0–F)
- `BBBBBBBB` - 8-bit arrow pattern (1 = arrow present), in order: N NE E SE S SW W NW

Example: `2P34 10110101` is a Physical card with attack 2, physical defence 3, magic defence 4, and arrows pointing N, E, SE, SW, NW.

### Workflow

2. Press `b` to mark any blocked cells
3. Press `i` to add each card in your hand
4. Press `e` to place any opponent cards already on the board
5. Press `Space` to solve - the adviser highlights the recommended card and cell
6. Press `p` to place the card, or choose a different one with `[`/`]` and place manually

## How it works

The adviser uses a **minimax search** (depth 3, with alpha-beta pruning) to evaluate possible placements. Blue's moves are simulated using your actual hand; Red's responses are modelled with a conservative proxy card. The score is the net difference in card ownership after the search tree is evaluated.

Battle resolution follows the Tetra Master rules: each card stat is treated as a hex digit representing the upper bound of a uniform random roll, with the midpoint used for expected-value calculations. Physical cards attack physical defence, Magic cards attack magic defence, Flexible cards attack whichever is lower, and Assault cards attack the minimum of physical defence, magic defence, and the defender's own attack.

## License

[GPL-3.0](https://github.com/bengosney/tetra-master-adviser/blob/main/LICENSE)
