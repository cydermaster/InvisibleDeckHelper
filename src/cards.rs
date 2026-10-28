// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Cydermaster

//! The Invisible Deck pairing scheme.
//!
//! Every card is glued back-to-back with exactly one other card, so the deck is
//! 26 pairs. The published arrangement (Royal Magic / FUN Inc. pairing guide)
//! is:
//!
//! * **Hearts pair with Spades, Diamonds pair with Clubs** — always one red and
//!   one black card.
//! * **Each pair totals 13** — Ace (1) with Queen (12), 2 with Jack, 3 with 10,
//!   4 with 9, 5 with 8, 6 with 7 — and because each rank appears in both suit
//!   orders, all 52 cards are covered.
//! * **Kings pair with Kings** (13 + 13 is not 13, so they are the exception).
//!
//! So the partner of a card is: same rank-complement, opposite colour suit.
//! This is an involution: the partner's partner is the original card.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Spades, Suit::Hearts, Suit::Diamonds, Suit::Clubs];

    pub fn symbol(self) -> &'static str {
        match self {
            Suit::Spades => "\u{2660}",
            Suit::Hearts => "\u{2665}",
            Suit::Diamonds => "\u{2666}",
            Suit::Clubs => "\u{2663}",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Suit::Spades => "Spades",
            Suit::Hearts => "Hearts",
            Suit::Diamonds => "Diamonds",
            Suit::Clubs => "Clubs",
        }
    }

    pub fn is_red(self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }

    /// Hearts <-> Spades, Diamonds <-> Clubs.
    pub fn partner(self) -> Suit {
        match self {
            Suit::Hearts => Suit::Spades,
            Suit::Spades => Suit::Hearts,
            Suit::Diamonds => Suit::Clubs,
            Suit::Clubs => Suit::Diamonds,
        }
    }
}

/// Short rank labels, Ace low (1) through King (13).
pub const RANK_SHORT: [&str; 13] =
    ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];

pub const RANK_LONG: [&str; 13] = [
    "Ace",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "10",
    "Jack",
    "Queen",
    "King",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Card {
    /// 1 = Ace ... 13 = King.
    pub rank: u8,
    pub suit: Suit,
}

impl Card {
    pub fn new(rank: u8, suit: Suit) -> Card {
        assert!((1..=13).contains(&rank), "rank must be 1..=13");
        Card { rank, suit }
    }

    /// The card glued to this one's back in an Invisible Deck.
    pub fn partner(self) -> Card {
        // Kings pair with Kings; everything else pairs to 13 minus its rank.
        let rank = if self.rank == 13 { 13 } else { 13 - self.rank };
        Card { rank, suit: self.suit.partner() }
    }

    pub fn short(self) -> String {
        format!("{}{}", RANK_SHORT[(self.rank - 1) as usize], self.suit.symbol())
    }

    pub fn long(self) -> String {
        format!(
            "{} of {}",
            RANK_LONG[(self.rank - 1) as usize],
            self.suit.name()
        )
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short())
    }
}

/// All 52 cards, ordered suit-major like the published table.
pub fn full_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for suit in Suit::ALL {
        for rank in 1..=13u8 {
            deck.push(Card::new(rank, suit));
        }
    }
    deck
}

/// The 26 pairs, each listed once, as (lower-half card, its partner).
pub fn all_pairs() -> Vec<(Card, Card)> {
    let mut seen: Vec<Card> = Vec::new();
    let mut pairs = Vec::with_capacity(26);
    for card in full_deck() {
        if seen.contains(&card) {
            continue;
        }
        let partner = card.partner();
        seen.push(card);
        seen.push(partner);
        pairs.push((card, partner));
    }
    pairs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Spot checks transcribed directly from the Royal Magic / FUN Inc.
    /// Invisible Deck pairing guide.
    #[test]
    fn matches_the_published_table() {
        let cases = [
            (Card::new(1, Suit::Hearts), Card::new(12, Suit::Spades)), // A♥ / Q♠
            (Card::new(3, Suit::Hearts), Card::new(10, Suit::Spades)), // 3♥ / 10♠
            (Card::new(5, Suit::Hearts), Card::new(8, Suit::Spades)),  // 5♥ / 8♠
            (Card::new(7, Suit::Hearts), Card::new(6, Suit::Spades)),  // 7♥ / 6♠
            (Card::new(9, Suit::Hearts), Card::new(4, Suit::Spades)),  // 9♥ / 4♠
            (Card::new(11, Suit::Hearts), Card::new(2, Suit::Spades)), // J♥ / 2♠
            (Card::new(13, Suit::Hearts), Card::new(13, Suit::Spades)), // K♥ / K♠
            (Card::new(1, Suit::Spades), Card::new(12, Suit::Hearts)), // A♠ / Q♥
            (Card::new(3, Suit::Spades), Card::new(10, Suit::Hearts)), // 3♠ / 10♥
            (Card::new(1, Suit::Diamonds), Card::new(12, Suit::Clubs)), // A♦ / Q♣
            (Card::new(7, Suit::Diamonds), Card::new(6, Suit::Clubs)), // 7♦ / 6♣
            (Card::new(13, Suit::Diamonds), Card::new(13, Suit::Clubs)), // K♦ / K♣
            (Card::new(1, Suit::Clubs), Card::new(12, Suit::Diamonds)), // A♣ / Q♦
            (Card::new(7, Suit::Clubs), Card::new(6, Suit::Diamonds)), // 7♣ / 6♦
            (Card::new(11, Suit::Clubs), Card::new(2, Suit::Diamonds)), // J♣ / 2♦
        ];
        for (a, b) in cases {
            assert_eq!(a.partner(), b, "{a} should pair with {b}");
            assert_eq!(b.partner(), a, "{b} should pair with {a}");
        }
    }

    #[test]
    fn pairing_is_an_involution() {
        for card in full_deck() {
            assert_eq!(
                card.partner().partner(),
                card,
                "partner of partner of {card} must be {card}"
            );
        }
    }

    #[test]
    fn no_card_pairs_with_itself() {
        for card in full_deck() {
            assert_ne!(card.partner(), card, "{card} must not pair with itself");
        }
    }

    #[test]
    fn every_pair_is_one_red_one_black() {
        for card in full_deck() {
            assert_ne!(
                card.suit.is_red(),
                card.partner().suit.is_red(),
                "{card} and its partner must be opposite colours"
            );
        }
    }

    #[test]
    fn non_kings_sum_to_thirteen() {
        for card in full_deck() {
            if card.rank == 13 {
                assert_eq!(card.partner().rank, 13, "kings pair with kings");
            } else {
                assert_eq!(
                    card.rank as u16 + card.partner().rank as u16,
                    13,
                    "{card} plus its partner should total 13"
                );
            }
        }
    }

    #[test]
    fn there_are_exactly_twenty_six_pairs_covering_all_52_cards() {
        let pairs = all_pairs();
        assert_eq!(pairs.len(), 26);
        let mut covered: Vec<Card> = Vec::new();
        for (a, b) in &pairs {
            covered.push(*a);
            covered.push(*b);
        }
        covered.sort();
        covered.dedup();
        assert_eq!(covered.len(), 52, "all 52 cards must appear exactly once");
        let mut deck = full_deck();
        deck.sort();
        assert_eq!(covered, deck);
    }

    #[test]
    fn suit_pairings_are_cross_colour() {
        assert_eq!(Suit::Hearts.partner(), Suit::Spades);
        assert_eq!(Suit::Spades.partner(), Suit::Hearts);
        assert_eq!(Suit::Diamonds.partner(), Suit::Clubs);
        assert_eq!(Suit::Clubs.partner(), Suit::Diamonds);
    }
}
