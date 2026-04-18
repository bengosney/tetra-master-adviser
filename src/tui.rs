use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell as RCell, Paragraph, Row, Table},
};
use std::io;

use crate::board::{Board, Cell, Owner};
use crate::card::{
    ARROW_E, ARROW_N, ARROW_NE, ARROW_NW, ARROW_S, ARROW_SE, ARROW_SW, ARROW_W, Card,
};
use crate::solver::{Move, best_move};
use crate::state::{SaveState, load, save};

pub struct App {
    pub board: Board,
    pub hand: Vec<Card>,
    pub best: Option<Move>,
    pub input_mode: InputMode,
    pub input_buf: String,
    pub status_msg: String,
    pub cursor: (usize, usize),
    pub selected_hand: usize,
}

#[derive(PartialEq, Eq)]
pub enum InputMode {
    Normal,
    EnteringCard { target: CardTarget },
}

#[derive(PartialEq, Eq)]
pub enum CardTarget {
    Hand,
    Board {
        row: usize,
        col: usize,
        owner: Owner,
    },
}

impl App {
    pub fn new() -> Self {
        Self {
            board: Board::new(),
            hand: Vec::new(),
            best: None,
            input_mode: InputMode::Normal,
            input_buf: String::new(),
            status_msg: String::new(),
            cursor: (0, 0),
            selected_hand: 0,
        }
    }

    pub fn from_save(s: SaveState) -> Self {
        let mut app = Self::new();
        app.board = s.board;
        app.hand = s.hand;
        app.cursor = (s.cursor.0.min(3), s.cursor.1.min(3));
        app.status_msg = "State restored.".into();
        app
    }

    pub fn to_save_state(&self) -> SaveState {
        SaveState {
            board: self.board.clone(),
            hand: self.hand.clone(),
            cursor: self.cursor,
        }
    }

    pub fn solve(&mut self) {
        if self.hand.is_empty() {
            self.status_msg = "No cards in hand.".into();
            return;
        }
        self.best = best_move(&self.board, &self.hand);
        match &self.best {
            None => self.status_msg = "No moves available.".into(),
            Some(m) => {
                self.selected_hand = m.card_index;
                self.cursor = (m.row, m.col);
                self.status_msg = format!(
                    "Best move: card {} ({}) → ({},{})  net gain: {}  — press p to place",
                    m.card_index + 1,
                    self.hand[m.card_index].stat_string(),
                    m.row,
                    m.col,
                    m.score
                );
            }
        }
    }
}

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = match load() {
        Some(s) => App::from_save(s),
        None => App::new(),
    };
    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let _ = save(&SaveState {
        board: app.board,
        hand: app.hand,
        cursor: app.cursor,
    });

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match &app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Left => app.cursor.1 = app.cursor.1.saturating_sub(1),
                    KeyCode::Right => app.cursor.1 = (app.cursor.1 + 1).min(3),
                    KeyCode::Up => app.cursor.0 = app.cursor.0.saturating_sub(1),
                    KeyCode::Down => app.cursor.0 = (app.cursor.0 + 1).min(3),
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::EnteringCard {
                            target: CardTarget::Hand,
                        };
                        app.input_buf.clear();
                        app.status_msg = "Enter card (e.g. 2P34 10110101):".into();
                    }
                    KeyCode::Char('e') => {
                        let (r, c) = app.cursor;
                        app.input_mode = InputMode::EnteringCard {
                            target: CardTarget::Board {
                                row: r,
                                col: c,
                                owner: Owner::Red,
                            },
                        };
                        app.input_buf.clear();
                        app.status_msg = format!("Place opponent card at ({r},{c}) — enter stats:");
                    }
                    KeyCode::Char('b') => {
                        let (r, c) = app.cursor;
                        app.board.set(r, c, crate::board::Cell::Blocked);
                        app.status_msg = format!("Blocked ({r},{c})");
                        let _ = save(&app.to_save_state());
                    }
                    KeyCode::Char(' ') => app.solve(),
                    KeyCode::Char('[') => {
                        if !app.hand.is_empty() {
                            app.selected_hand = app.selected_hand.saturating_sub(1);
                        }
                    }
                    KeyCode::Char(']') => {
                        if !app.hand.is_empty() {
                            app.selected_hand = (app.selected_hand + 1).min(app.hand.len() - 1);
                        }
                    }
                    KeyCode::Char('f') => {
                        let (r, c) = app.cursor;
                        if let Cell::Occupied { card, owner } = app.board.cell(r, c) {
                            let new_owner = if owner == Owner::Blue {
                                Owner::Red
                            } else {
                                Owner::Blue
                            };
                            app.board.set(
                                r,
                                c,
                                Cell::Occupied {
                                    card,
                                    owner: new_owner,
                                },
                            );
                            app.best = None;
                            app.status_msg = format!(
                                "Flipped ({r},{c}) to {}.",
                                if new_owner == Owner::Blue {
                                    "Blue"
                                } else {
                                    "Red"
                                }
                            );
                            let _ = save(&app.to_save_state());
                        } else {
                            app.status_msg = "No card at cursor to flip.".into();
                        }
                    }
                    KeyCode::Char('p') => {
                        let (r, c) = app.cursor;
                        if app.hand.is_empty() {
                            app.status_msg = "No cards in hand.".into();
                        } else if !matches!(app.board.cell(r, c), Cell::Empty) {
                            app.status_msg = format!("Cell ({r},{c}) is not empty.");
                        } else {
                            let idx = app.selected_hand.min(app.hand.len() - 1);
                            let card = app.hand.remove(idx);
                            app.selected_hand = idx.min(app.hand.len().saturating_sub(1));
                            app.board.set(
                                r,
                                c,
                                Cell::Occupied {
                                    card,
                                    owner: Owner::Blue,
                                },
                            );
                            app.best = None;
                            app.status_msg = format!("Placed {} at ({r},{c}).", card.stat_string());
                            let _ = save(&app.to_save_state());
                        }
                    }
                    KeyCode::Char('r') => {
                        app.board = Board::new();
                        app.hand.clear();
                        app.best = None;
                        app.status_msg = "Board reset.".into();
                        let _ = save(&app.to_save_state());
                    }
                    _ => {}
                },
                InputMode::EnteringCard { .. } => match key.code {
                    KeyCode::Char(c) => app.input_buf.push(c),
                    KeyCode::Backspace => {
                        app.input_buf.pop();
                    }
                    KeyCode::Enter => {
                        let buf = app.input_buf.trim().to_string();
                        match Card::parse(&buf) {
                            Err(e) => {
                                app.status_msg = format!("Parse error: {e}");
                            }
                            Ok(card) => {
                                let mode =
                                    std::mem::replace(&mut app.input_mode, InputMode::Normal);
                                match mode {
                                    InputMode::EnteringCard {
                                        target: CardTarget::Hand,
                                    } => {
                                        app.hand.push(card);
                                        app.status_msg = format!(
                                            "Added {} to hand ({} cards)",
                                            card.stat_string(),
                                            app.hand.len()
                                        );
                                        let _ = save(&app.to_save_state());
                                    }
                                    InputMode::EnteringCard {
                                        target: CardTarget::Board { row, col, owner },
                                    } => {
                                        app.board.set(row, col, Cell::Occupied { card, owner });
                                        app.status_msg = format!(
                                            "Placed {} at ({row},{col})",
                                            card.stat_string()
                                        );
                                        let _ = save(&app.to_save_state());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        app.input_buf.clear();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                        app.input_buf.clear();
                        app.status_msg = "Cancelled.".into();
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(22), // board + controls (4 rows × 5 lines + 2 outer border)
            Constraint::Length(9),  // hand (5 cards + header + 2 borders)
            Constraint::Length(3),  // input / status
            Constraint::Min(0),
        ])
        .split(area);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Min(0)])
        .split(chunks[0]);

    draw_board(f, app, top[0]);
    draw_controls(f, top[1]);
    draw_hand(f, app, chunks[1]);
    draw_status(f, app, chunks[2]);
}

fn draw_board(f: &mut Frame, app: &App, area: Rect) {
    let cell_w = 10u16;
    let cell_h = 5u16; // 3 content lines + 2 border lines

    let block = Block::default().title("Board (4×4)").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    for row in 0..4usize {
        for col in 0..4usize {
            let x = inner.x + col as u16 * cell_w;
            let y = inner.y + row as u16 * cell_h;
            if x + cell_w > inner.x + inner.width || y + cell_h > inner.y + inner.height {
                continue;
            }
            let rect = Rect::new(x, y, cell_w, cell_h);
            let is_cursor = app.cursor == (row, col);
            let is_best = app
                .best
                .as_ref()
                .is_some_and(|m| m.row == row && m.col == col);

            let border_style = if is_cursor {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_best {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if matches!(app.board.cell(row, col), Cell::Blocked) {
                Style::default().fg(Color::Rgb(120, 60, 60))
            } else if matches!(
                app.board.cell(row, col),
                Cell::Occupied {
                    owner: Owner::Red,
                    ..
                }
            ) {
                Style::default().fg(Color::Red)
            } else if matches!(
                app.board.cell(row, col),
                Cell::Occupied {
                    owner: Owner::Blue,
                    ..
                }
            ) {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let (top_line, mid_line, bot_line, content_style) = match app.board.cell(row, col) {
                Cell::Empty => {
                    let style = if is_best {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };
                    (
                        "       ".to_string(),
                        "   ·   ".to_string(),
                        "       ".to_string(),
                        style,
                    )
                }
                Cell::Blocked => (
                    "▓▓▓▓▓▓▓".to_string(),
                    "▓▓▓▓▓▓▓".to_string(),
                    "▓▓▓▓▓▓▓".to_string(),
                    Style::default().fg(Color::Rgb(120, 60, 60)),
                ),
                Cell::Occupied { card, owner } => {
                    let color = if owner == Owner::Blue {
                        Color::Cyan
                    } else {
                        Color::Red
                    };
                    let style = if is_best {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(color)
                    };
                    let [top, mid, bot] = arrow_grid(card.arrows);
                    let stat = card.stat_string();
                    (top, format!("{mid} {stat}"), bot, style)
                }
            };

            let bg = if is_cursor {
                Color::Rgb(40, 40, 60)
            } else if matches!(app.board.cell(row, col), Cell::Blocked) {
                Color::Rgb(50, 25, 25)
            } else if matches!(
                app.board.cell(row, col),
                Cell::Occupied {
                    owner: Owner::Red,
                    ..
                }
            ) {
                Color::Rgb(50, 20, 20)
            } else if matches!(
                app.board.cell(row, col),
                Cell::Occupied {
                    owner: Owner::Blue,
                    ..
                }
            ) {
                Color::Rgb(20, 40, 50)
            } else {
                Color::Reset
            };
            let text_style = content_style.bg(bg);

            let para = Paragraph::new(vec![
                Line::from(Span::styled(format!("{:<8}", top_line), text_style)),
                Line::from(Span::styled(format!("{:<8}", mid_line), text_style)),
                Line::from(Span::styled(format!("{:<8}", bot_line), text_style)),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );
            f.render_widget(para, rect);
        }
    }
}

/// Returns [top, mid, bot] arrow rows for display across 3 lines.
/// top: ↖ ↑ ↗
/// mid: ←   →
/// bot: ↙ ↓ ↘
fn arrow_grid(arrows: u8) -> [String; 3] {
    let ch = |bit: u8, sym: char| if arrows & bit != 0 { sym } else { ' ' };
    let top = format!(
        "{}{}{}",
        ch(ARROW_NW, '↖'),
        ch(ARROW_N, '↑'),
        ch(ARROW_NE, '↗'),
    );
    let mid = format!("{} {}", ch(ARROW_W, '←'), ch(ARROW_E, '→'),);
    let bot = format!(
        "{}{}{}",
        ch(ARROW_SW, '↙'),
        ch(ARROW_S, '↓'),
        ch(ARROW_SE, '↘'),
    );
    [top, mid, bot]
}

fn draw_hand(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().title("Your Hand").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.hand.is_empty() {
        f.render_widget(Paragraph::new("(empty)  Press 'i' to add a card"), inner);
        return;
    }

    let selected = app.selected_hand.min(app.hand.len().saturating_sub(1));
    let rows: Vec<Row> = app
        .hand
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let is_best = app.best.as_ref().is_some_and(|m| m.card_index == i);
            let is_selected = i == selected;
            let style = if is_best {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Rgb(40, 40, 60))
            } else {
                Style::default().fg(Color::Cyan)
            };
            Row::new(vec![
                RCell::from(format!("{}", i + 1)),
                RCell::from(c.stat_string()),
                RCell::from(c.arrow_display()),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),
            Constraint::Length(6),
            Constraint::Min(10),
        ],
    )
    .header(
        Row::new(["#", "Stats", "Arrows"]).style(Style::default().add_modifier(Modifier::BOLD)),
    );
    f.render_widget(table, inner);
}

fn draw_controls(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(vec![Span::styled(
            "Controls",
            Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑↓←→     ", Style::default().fg(Color::Yellow)),
            Span::raw("move cursor"),
        ]),
        Line::from(vec![
            Span::styled("i        ", Style::default().fg(Color::Yellow)),
            Span::raw("add card to hand"),
        ]),
        Line::from(vec![
            Span::styled("[/]      ", Style::default().fg(Color::Yellow)),
            Span::raw("select hand card"),
        ]),
        Line::from(vec![
            Span::styled("p        ", Style::default().fg(Color::Yellow)),
            Span::raw("place selected card"),
        ]),
        Line::from(vec![
            Span::styled("e        ", Style::default().fg(Color::Yellow)),
            Span::raw("place opponent card"),
        ]),
        Line::from(vec![
            Span::styled("f        ", Style::default().fg(Color::Yellow)),
            Span::raw("flip card colour"),
        ]),
        Line::from(vec![
            Span::styled("b        ", Style::default().fg(Color::Yellow)),
            Span::raw("block cell"),
        ]),
        Line::from(vec![
            Span::styled("Space    ", Style::default().fg(Color::Yellow)),
            Span::raw("solve best move"),
        ]),
        Line::from(vec![
            Span::styled("r        ", Style::default().fg(Color::Yellow)),
            Span::raw("reset board"),
        ]),
        Line::from(vec![
            Span::styled("q        ", Style::default().fg(Color::Yellow)),
            Span::raw("quit & save"),
        ]),
    ];
    let para = Paragraph::new(lines).block(Block::default().title("Help").borders(Borders::ALL));
    f.render_widget(para, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let content = if matches!(app.input_mode, InputMode::EnteringCard { .. }) {
        format!("{} > {}_", app.status_msg, app.input_buf)
    } else {
        app.status_msg.clone()
    };
    let para =
        Paragraph::new(content).block(Block::default().title("Status").borders(Borders::ALL));
    f.render_widget(para, area);
}
