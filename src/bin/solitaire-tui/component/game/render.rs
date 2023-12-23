use std::collections::HashMap;

use ratatui::{layout::Rect, prelude::*, symbols::*, text::Text, widgets::*, Frame};
use solitaire::{variant::klondike, GameState as GameStateTrait};

use crate::component::game::ui_state::{MovingState, SelectingState, UIState};

const CARD_WIDTH: u16 = 10;
const CARD_HEIGHT: u16 = 7;

/// The render states a card can be in
#[derive(Copy, Clone, Eq, PartialEq)]
enum CardState {
    Normal,
    Selected,
    Moving,
}

/// A [Card](klondike::Card) with its [CardState] for rendering
type Card = (klondike::Card, CardState);

/// Represents a GameState as it should be rendered
pub struct GameState {
    piles: HashMap<klondike::PileRef, (Vec<Card>, bool)>,
}

impl From<(&klondike::GameStateOption, &UIState)> for GameState {
    fn from((game_state, ui_state): (&klondike::GameStateOption, &UIState)) -> Self {
        let pile_refs = [klondike::PileRef::Stock, klondike::PileRef::Talon]
            .iter()
            .cloned()
            .chain(
                [(); klondike::NUM_FOUNDATIONS]
                    .iter()
                    .enumerate()
                    .map(|(i, _)| klondike::PileRef::Foundation(i)),
            )
            .chain(
                [(); klondike::NUM_TABLEAU]
                    .iter()
                    .enumerate()
                    .map(|(i, _)| klondike::PileRef::Tableau(i)),
            )
            .collect::<Vec<_>>();

        let mut piles = HashMap::with_capacity(pile_refs.len());

        for p in pile_refs {
            let is_selected = match ui_state {
                UIState::Hovering(pile_ref) => pile_ref == &p,
                _ => false,
            };
            let stack = game_state.get_stack(p).map_or_else(
                || Vec::new(),
                |s| s.iter().cloned().map(|c| (c, CardState::Normal)).collect(),
            );
            piles.insert(p, (stack, is_selected));
        }

        // Set the CardState correctly
        match ui_state {
            UIState::Hovering(pile_ref) => {
                let (ref mut pile, _) = piles.get_mut(&pile_ref).unwrap();
                match pile.last_mut() {
                    Some((_, s)) => *s = CardState::Selected,
                    _ => {}
                }
            }
            UIState::Selecting(SelectingState::Tableau { pile_n, take_n }) => {
                let (ref mut pile, _) =
                    piles.get_mut(&klondike::PileRef::Tableau(*pile_n)).unwrap();
                let pile_len = pile.len();
                for (_, s) in &mut pile[pile_len - take_n..pile_len] {
                    *s = CardState::Selected;
                }
            }
            UIState::Moving(MovingState { src, take_n, dst }) => {
                let (ref mut src, _) = piles.get_mut(&src).unwrap();
                let mut take = solitaire::take_n_vec_mut(src, *take_n);
                for (_, s) in &mut take {
                    *s = CardState::Moving;
                }
                let (ref mut dst, _) = piles.get_mut(&dst).unwrap();
                *dst = dst.iter().chain(take.iter()).cloned().collect()
            }
        }

        GameState { piles }
    }
}

impl GameState {
    pub fn render(&self, f: &mut Frame, rect: Rect) {
        let width = CARD_WIDTH * klondike::NUM_TABLEAU as u16;
        let horizontal_pad = rect.width.checked_sub(width).unwrap_or(0);

        let inner_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(width), Constraint::Min(horizontal_pad)])
            .split(rect)[0];

        let vstack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(CARD_HEIGHT),
                Constraint::Length(CARD_HEIGHT * 3),
            ])
            .split(inner_rect);

        // Render the top row
        {
            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(CARD_WIDTH),
                    // Talon is two widths wide
                    Constraint::Length(CARD_WIDTH * 2),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                    Constraint::Length(CARD_WIDTH),
                ])
                .split(vstack[0]);

            self.render_stock(f, top[0]);
            self.render_talon(f, top[1]);

            for (i, foundation_rect) in top[2..6].iter().cloned().enumerate() {
                self.render_foundation(i, f, foundation_rect);
            }
        }

        // Render the tableau
        let tableau = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([(); klondike::NUM_TABLEAU].map(|_| Constraint::Length(CARD_WIDTH)))
            .split(vstack[1]);

        for (i, tableau_rect) in tableau.iter().cloned().enumerate() {
            self.render_tableau(i, f, tableau_rect);
        }
    }

    fn render_stock(&self, f: &mut Frame, rect: Rect) {
        let (pile, is_selected) = self.piles.get(&klondike::PileRef::Stock).unwrap();

        render_card(pile.last(), *is_selected, border::ROUNDED, f, rect);
    }

    fn render_talon(&self, f: &mut Frame, rect: Rect) {
        let (pile, is_selected) = self.piles.get(&klondike::PileRef::Talon).unwrap();

        // todo handle 3 card draw

        let rx_padding = rect.width.checked_sub(CARD_WIDTH).unwrap_or(0);

        let rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(CARD_WIDTH), Constraint::Min(rx_padding)])
            .split(rect);

        render_card(pile.last(), *is_selected, border::ROUNDED, f, rect[0]);
    }

    fn render_foundation(&self, i: usize, f: &mut Frame, rect: Rect) {
        let (pile, is_selected) = self.piles.get(&klondike::PileRef::Foundation(i)).unwrap();

        render_card(pile.last(), *is_selected, border::ROUNDED, f, rect);
    }

    fn render_tableau(&self, i: usize, f: &mut Frame, rect: Rect) {
        let (pile, is_selected) = self.piles.get(&klondike::PileRef::Tableau(i)).unwrap();

        if pile.is_empty() {
            let by_padding = rect.height.checked_sub(CARD_HEIGHT).unwrap_or(0);

            let rect = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(CARD_HEIGHT), Constraint::Min(by_padding)])
                .split(rect);

            return render_card(None, *is_selected, border::ROUNDED, f, rect[0]);
        }

        let mut ty_padding = 0;
        for (i, &(c, s)) in pile.iter().enumerate() {
            let border_set: border::Set = if i != 0 {
                border::Set {
                    top_left: line::VERTICAL_RIGHT,
                    top_right: line::VERTICAL_LEFT,
                    ..border::ROUNDED
                }
            } else {
                border::ROUNDED
            };

            let by_padding = rect
                .height
                .checked_sub(ty_padding + CARD_HEIGHT)
                .unwrap_or(0);

            let rect = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(ty_padding),
                    Constraint::Length(CARD_HEIGHT),
                    Constraint::Min(by_padding),
                ])
                .split(rect);

            render_card(Some(&(c, s)), false, border_set, f, rect[1]);

            // Add 2 to the padding if the card is face up so the suit and rank are visible
            if c.face_up {
                ty_padding += 2

                // Only use 1 padding for face down cards to minimise space
            } else {
                ty_padding += 1
            }
        }
    }
}

fn render_card(
    card: Option<&Card>,
    is_selected: bool,
    border_set: border::Set,
    f: &mut Frame,
    rect: Rect,
) {
    let state = card.map(|(_, s)| s).unwrap_or_else(|| {
        if is_selected {
            &CardState::Selected
        } else {
            &CardState::Normal
        }
    });

    let border_color = if state == &CardState::Selected {
        Color::LightGreen
    } else if state == &CardState::Moving {
        Color::LightYellow
    } else {
        Color::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .fg(border_color);

    let inner_rect = block.inner(rect);

    match card {
        Some((c, _)) => match c.face_up {
            true => f.render_widget(
                Paragraph::new(Text::styled(
                    card_to_str(c, inner_rect),
                    Style::default().bg(Color::White).fg(card_to_color(c)),
                ))
                .block(block),
                rect,
            ),
            false => f.render_widget(
                Paragraph::new(Text::styled(
                    card_back_str(inner_rect),
                    Style::default().bg(Color::Red).fg(Color::LightRed),
                ))
                .block(block),
                rect,
            ),
        },
        None => f.render_widget(block, rect),
    }
}

fn rank_to_str(r: klondike::Rank) -> String {
    format!(
        "{:<2}",
        match r {
            klondike::Rank::King => "K",
            klondike::Rank::Queen => "Q",
            klondike::Rank::Jack => "J",
            klondike::Rank::Ten => "10",
            klondike::Rank::Nine => "9",
            klondike::Rank::Eight => "8",
            klondike::Rank::Seven => "7",
            klondike::Rank::Six => "6",
            klondike::Rank::Five => "5",
            klondike::Rank::Four => "4",
            klondike::Rank::Three => "3",
            klondike::Rank::Two => "2",
            klondike::Rank::Ace => "A",
        }
    )
}

fn suit_to_str(s: klondike::FrenchSuit) -> &'static str {
    match s {
        klondike::FrenchSuit::Clubs => "♣",
        klondike::FrenchSuit::Spades => "♠",
        klondike::FrenchSuit::Hearts => "♥",
        klondike::FrenchSuit::Diamonds => "♦",
    }
}

fn card_to_str(c: &klondike::Card, rect: Rect) -> String {
    (0..rect.height)
        .map(|i| {
            let r = rank_to_str(c.rank);
            let s = suit_to_str(c.suit);
            if i == 0 {
                format!("{}{:>w$}", s, r, w = rect.width as usize - 1)
            } else if i == rect.height - 1 {
                format!("{}{:>w$}", r, s, w = rect.width as usize - 2)
            } else {
                " ".repeat(rect.width as usize)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn card_back_str(rect: Rect) -> String {
    (0..rect.height)
        .map(|i| if i % 2 == 0 { " #" } else { "# " }.repeat((rect.width as usize) / 2))
        .collect::<Vec<_>>()
        .join("\n")
}

fn card_to_color(c: &klondike::Card) -> Color {
    if c.suit.color() == klondike::Color::Red {
        Color::Red
    } else {
        Color::DarkGray
    }
}
