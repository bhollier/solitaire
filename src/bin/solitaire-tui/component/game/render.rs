use ratatui::{layout::Rect, prelude::*, symbols::*, text::Text, widgets::*, Frame};
use solitaire::variant::klondike;

pub const CARD_WIDTH: u16 = 10;
pub const CARD_HEIGHT: u16 = 7;

pub fn stock(stock: &[klondike::Card], selected: bool, f: &mut Frame, rect: Rect) {
    render_card(stock.last(), selected, border::ROUNDED, f, rect)
}

pub fn talon(talon: &[klondike::Card], selected: bool, f: &mut Frame, rect: Rect) {
    // todo handle 3 card draw

    let rx_padding = rect.width.checked_sub(CARD_WIDTH).unwrap_or(0);

    let rect = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(CARD_WIDTH), Constraint::Min(rx_padding)])
        .split(rect);

    render_card(talon.last(), selected, border::ROUNDED, f, rect[0])
}

pub fn foundation(foundation: &[klondike::Card], selected: bool, f: &mut Frame, rect: Rect) {
    render_card(foundation.last(), selected, border::ROUNDED, f, rect)
}

#[derive(Eq, PartialEq)]
pub enum TableauSelected {
    Unselected,
    Selected(usize),
}

pub fn tableau(tableau: &[klondike::Card], selected: TableauSelected, f: &mut Frame, rect: Rect) {
    if tableau.is_empty() {
        let by_padding = rect.height.checked_sub(CARD_HEIGHT).unwrap_or(0);

        let rect = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(CARD_HEIGHT), Constraint::Min(by_padding)])
            .split(rect);

        return render_card(
            None,
            selected != TableauSelected::Unselected,
            border::ROUNDED,
            f,
            rect[0],
        );
    }

    let mut ty_padding = 0;
    for (i, c) in tableau.iter().enumerate() {
        let is_selected = match selected {
            TableauSelected::Unselected => false,
            TableauSelected::Selected(take_n) => tableau.len() - i <= take_n,
        };

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

        render_card(Some(c), is_selected, border_set, f, rect[1]);

        // Add 2 to the padding if the card is face up so the suit and rank are visible
        if c.face_up {
            ty_padding += 2

            // Only use 1 padding for face down cards to minimise space
        } else {
            ty_padding += 1
        }
    }
}

fn render_card(
    card: Option<&klondike::Card>,
    selected: bool,
    border_set: border::Set,
    f: &mut Frame,
    rect: Rect,
) {
    let border_color = if selected {
        Color::LightGreen
    } else {
        Color::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .fg(border_color);

    let inner_rect = block.inner(rect);

    match card {
        Some(c) => match c.face_up {
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
