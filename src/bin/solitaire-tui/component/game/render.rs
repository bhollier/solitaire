use std::collections::HashMap;
use std::rc::Rc;

use ratatui::{layout::Flex, layout::Rect, prelude::*, symbols::*, text::Text, widgets::*, Frame};
use solitaire::{variant::klondike, GameState as GameStateTrait};

use crate::component::game::ui_state::{MovingState, SelectingState, UIState};

const CARD_WIDTH: u16 = 10;
const CARD_HEIGHT: u16 = 7;
const TOTAL_WIDTH: u16 = CARD_WIDTH * klondike::NUM_TABLEAU as u16;

/// The render states a card can be in
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CardVisualState {
    Normal,
    Selected,
    Moving,
}

pub enum CardLocation {
    Tableau(usize, usize),
    Foundation(usize),
    Stock,
    Talon,
}

impl CardLocation {
    pub fn pile_ref(&self) -> klondike::PileRef {
        match self {
            CardLocation::Tableau(p, _) => klondike::PileRef::Tableau(*p),
            CardLocation::Foundation(p) => klondike::PileRef::Foundation(*p),
            CardLocation::Stock => klondike::PileRef::Stock,
            CardLocation::Talon => klondike::PileRef::Talon,
        }
    }

    pub fn n_from_bottom(&self) -> Option<usize> {
        match self {
            CardLocation::Tableau(_, n) => Some(*n),
            _ => None,
        }
    }
}

/// Render-specific information about a card
pub struct CardInfo {
    pub location: CardLocation,
    visual_state: CardVisualState,
    border: border::Set,
    rect: Rect,
    z: u64,
}

/// Represents the render state for a game
pub struct RenderState {
    rect: Rect,
    // List of cards for drawing, ordered by Z index
    draw_list: Vec<(Option<klondike::Card>, CardInfo)>,
    is_win: bool,
}

impl RenderState {
    pub fn new(game_state: &klondike::GameStateOption, ui_state: &UIState, rect: Rect) -> Self {
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

        // Construct a simple hash map of piles with the visual states of each card
        // according to the current game state
        let mut piles = HashMap::with_capacity(pile_refs.len());
        let mut total_cards = 0;
        for p in pile_refs {
            let stack = game_state.get_stack(p).map_or_else(
                || Vec::new(),
                |s| {
                    s.iter()
                        .cloned()
                        // All cards are normal initially
                        .map(|c| (c, CardVisualState::Normal))
                        .collect()
                },
            );
            total_cards += stack.len();
            piles.insert(p, stack);
        }

        // Update and move around cards based on the UI state
        match ui_state {
            UIState::Dealing(_) => {}
            UIState::Hovering(pile_ref) => {
                let pile = piles.get_mut(&pile_ref).unwrap();
                match pile.last_mut() {
                    Some((_, s)) => *s = CardVisualState::Selected,
                    _ => {} // Hovering over empty piles is handled later
                }
            }
            UIState::Selecting(SelectingState::Tableau { pile_n, take_n }) => {
                let pile = piles.get_mut(&klondike::PileRef::Tableau(*pile_n)).unwrap();
                let pile_len = pile.len();
                for (_, s) in &mut pile[pile_len - take_n..pile_len] {
                    *s = CardVisualState::Selected;
                }
            }
            UIState::Selecting(SelectingState::Talon) => {
                let pile = piles.get_mut(&klondike::PileRef::Talon).unwrap();
                let pile_len = pile.len();
                pile[pile_len - 1].1 = CardVisualState::Selected;
            }
            UIState::Moving(MovingState { src, take_n, dst }) => {
                if dst != src {
                    let src = piles.get_mut(&src).unwrap();
                    let mut take = solitaire::take_n_vec_mut(src, *take_n);
                    for (_, s) in &mut take {
                        *s = CardVisualState::Moving;
                    }
                    let dst = piles.get_mut(&dst).unwrap();
                    *dst = dst.iter().chain(take.iter()).cloned().collect()
                }
            }
            UIState::AutoMoving(_) => {}
        }

        // Divide the rect into sub-rects for each area of the game
        let layout = PileLayout::from(rect);

        // Construct the draw list of cards, with the total cards as a hint for the capacity
        // (the actual size will likely be a bit bigger than the total cards,
        // since the draw list also needs elements for empty piles)
        let mut draw_list = Vec::with_capacity(total_cards);

        // Closure for adding to the draw list for "one card piles",
        // i.e ones that are rendered as either having 1 or 0 cards
        let mut add_one_card_pile = |location: CardLocation, rect: Rect| {
            let pile = piles.get(&location.pile_ref()).unwrap();
            let (card, visual_state) = pile.last().map_or_else(
                // If there's no card, figure out the visual state from the UI state
                || match ui_state {
                    UIState::Hovering(p) if *p == location.pile_ref() => {
                        (None, CardVisualState::Selected)
                    }
                    _ => (None, CardVisualState::Normal),
                },
                |(c, visual_state)| (Some(*c), *visual_state),
            );
            draw_list.push((
                card,
                CardInfo {
                    location,
                    visual_state,
                    border: border::ROUNDED,
                    rect,
                    z: 0,
                },
            ));
        };

        add_one_card_pile(CardLocation::Stock, layout.stock);
        add_one_card_pile(CardLocation::Talon, layout.talon);
        for (i, rect) in layout.foundation.iter().cloned().enumerate() {
            add_one_card_pile(CardLocation::Foundation(i), rect);
        }

        // Closure for adding to the draw list for a tableau pile
        let mut add_tableau = |pile_n: usize, rect: Rect| {
            let pile = piles.get(&klondike::PileRef::Tableau(pile_n)).unwrap();
            if pile.is_empty() {
                let visual_state = match ui_state {
                    UIState::Hovering(p) if *p == klondike::PileRef::Tableau(pile_n) => {
                        CardVisualState::Selected
                    }
                    _ => CardVisualState::Normal,
                };

                let by_padding = rect.height.checked_sub(CARD_HEIGHT).unwrap_or(0);

                let rect = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(CARD_HEIGHT), Constraint::Min(by_padding)])
                    .split(rect);

                draw_list.push((
                    None,
                    CardInfo {
                        location: CardLocation::Tableau(pile_n, 0),
                        visual_state,
                        border: border::ROUNDED,
                        rect: rect[0],
                        z: 0,
                    },
                ));
            } else {
                let mut ty_padding = 0;
                for (i, &(c, visual_state)) in pile.iter().enumerate() {
                    let border: border::Set = if i != 0 {
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

                    draw_list.push((
                        Some(c),
                        CardInfo {
                            location: CardLocation::Tableau(pile_n, (pile.len() - 1) - i),
                            visual_state,
                            border,
                            rect: rect[1],
                            z: i as u64,
                        },
                    ));

                    // Add 2 to the padding if the card is face up so the suit and rank are visible
                    if c.face_up {
                        ty_padding += 2

                        // Only use 1 padding for face down cards to minimise space
                    } else {
                        ty_padding += 1
                    }
                }
            }
        };

        for (i, rect) in layout.tableau.iter().cloned().enumerate() {
            add_tableau(i, rect);
        }

        // Sort the draw list by Z index
        draw_list.sort_by_key(|(_, card_info)| card_info.z);

        RenderState {
            rect,
            draw_list,
            is_win: match game_state {
                klondike::GameStateOption::Win(_) => true,
                _ => false,
            },
        }
    }

    pub fn render(&self, f: &mut Frame) {
        for (card, card_info) in self.draw_list.iter() {
            render_card(card, card_info, f);
        }
        if self.is_win {
            let text = Text::raw("You win!");
            let [rect] = Layout::horizontal([Constraint::Length(text.width() as u16)])
                .flex(Flex::Center)
                .areas(self.rect);
            let [rect] = Layout::vertical([Constraint::Length(1)])
                .flex(Flex::Center)
                .areas(rect);
            f.render_widget(Clear, rect);
            f.render_widget(text, rect);
        }
    }

    pub fn find_card_at(&self, col: u16, row: u16) -> Option<&(Option<klondike::Card>, CardInfo)> {
        // Search in reverse order so that cards with a higher Z index are selected first
        self.draw_list.iter().rev().find(|(_, card_info)| {
            row >= card_info.rect.top()
                && row <= card_info.rect.bottom()
                && col >= card_info.rect.left()
                && col <= card_info.rect.right()
        })
    }
}

struct PileLayout {
    tableau: Rc<[Rect]>,
    foundation: Rc<[Rect]>,
    stock: Rect,
    talon: Rect,
}

impl From<Rect> for PileLayout {
    fn from(rect: Rect) -> Self {
        let padding = rect.width.checked_sub(TOTAL_WIDTH).unwrap_or(0) / 2;

        let inner_rect = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(padding),
                Constraint::Length(TOTAL_WIDTH),
                Constraint::Min(padding),
            ])
            .split(rect)[1];

        let vstack = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(CARD_HEIGHT), Constraint::Fill(1)])
            .split(inner_rect);

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

        let talon_rx_padding = top[1].width.checked_sub(CARD_WIDTH).unwrap_or(0);

        let talon = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(CARD_WIDTH),
                Constraint::Min(talon_rx_padding),
            ])
            .split(top[1])[0];

        let tableau = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([(); klondike::NUM_TABLEAU].map(|_| Constraint::Length(CARD_WIDTH)))
            .split(vstack[1]);

        PileLayout {
            tableau,
            foundation: Rc::from(&top[2..6]),
            stock: top[0],
            talon,
        }
    }
}

fn render_card(card: &Option<klondike::Card>, card_info: &CardInfo, f: &mut Frame) {
    let border_color = if card_info.visual_state == CardVisualState::Selected {
        Color::LightGreen
    } else if card_info.visual_state == CardVisualState::Moving {
        Color::LightYellow
    } else {
        Color::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(card_info.border)
        .fg(border_color);

    let inner_rect = block.inner(card_info.rect);

    match card {
        Some(c) => match c.face_up {
            true => f.render_widget(
                Paragraph::new(Text::styled(
                    card_to_str(&c, inner_rect),
                    Style::default().bg(Color::White).fg(card_to_color(c)),
                ))
                .block(block),
                card_info.rect,
            ),
            false => f.render_widget(
                Paragraph::new(Text::styled(
                    card_back_str(inner_rect),
                    Style::default().bg(Color::Red).fg(Color::LightRed),
                ))
                .block(block),
                card_info.rect,
            ),
        },
        None => f.render_widget(block, card_info.rect),
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
