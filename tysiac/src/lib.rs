use card_games_lib::{game_states, pile, step_try, Pile, Step, StepResult};
use core::convert::TryFrom;
use core::ops::Add;
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
use std::convert::TryInto;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct Game;

#[derive(EnumIter, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    A,
    B,
    C,
}

struct InfinitePlayerIter(Player);

impl Iterator for InfinitePlayerIter {
    type Item = Player;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        self.0 = self.0.next();
        Some(self.0)
    }
}

impl SomeState {
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        SomeState::Bidding(Bidding::random(rng))
    }

    pub fn deal(deck: &mut impl Iterator<Item = Card>) -> Self {
        SomeState::Bidding(Bidding::deal(deck))
    }
}

impl Bidding {
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let mut deck: Vec<_> = all_cards().collect();
        deck.shuffle(rng);

        let mut deck = deck.drain(..);

        Self::deal(&mut deck)
    }

    pub fn deal(deck: &mut impl Iterator<Item = Card>) -> Self {
        Self {
            hands: Piles::deal(deck),
            current_bid: (Player::A, Fives::one_hundred(), Player::B),
            prikup: [
                deck.next().unwrap(),
                deck.next().unwrap(),
                deck.next().unwrap(),
            ],
        }
    }
}

#[derive(EnumIter, Clone, Debug, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(EnumIter, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Nine,
    Jack,
    Queen,
    King,
    Ten,
    Ace,
}

fn all_cards() -> impl Iterator<Item = Card> {
    Rank::iter()
        .cartesian_product(Suit::iter())
        .map(|(r, s)| Card(r, s))
}

#[derive(Debug, PartialEq, Eq)]
// note that you cannot copy cards as these represent the physical deck
pub struct Card(Rank, Suit);

impl PartialEq<Card> for card_games_lib::Card {
    fn eq(&self, other: &Card) -> bool {
        other.description() == *self
    }
}

impl PartialEq<card_games_lib::Suit> for &Suit {
    fn eq(&self, other: &card_games_lib::Suit) -> bool {
        self.description() == *other
    }
}

impl PartialEq<card_games_lib::Rank> for &Rank {
    fn eq(&self, other: &card_games_lib::Rank) -> bool {
        self.description() == *other
    }
}

impl Card {
    pub fn suit(&self) -> &Suit {
        &self.1
    }

    pub fn rank(&self) -> &Rank {
        &self.0
    }

    pub fn description(&self) -> card_games_lib::Card {
        card_games_lib::Card(self.rank().description(), self.suit().description())
    }
}

impl Suit {
    pub fn marriage_value(&self) -> usize {
        match self {
            Suit::Hearts => 100,
            Suit::Diamonds => 80,
            Suit::Clubs => 60,
            Suit::Spades => 40,
        }
    }

    pub fn description(&self) -> card_games_lib::Suit {
        match self {
            Suit::Diamonds => card_games_lib::Suit::Diamonds,
            Suit::Clubs => card_games_lib::Suit::Clubs,
            Suit::Hearts => card_games_lib::Suit::Hearts,
            Suit::Spades => card_games_lib::Suit::Spades,
        }
    }
}

impl Rank {
    pub fn is_weddable(&self) -> bool {
        self == &Rank::King || self == &Rank::Queen
    }

    pub fn description(&self) -> card_games_lib::Rank {
        match self {
            Rank::Ace => card_games_lib::Rank::Ace,
            Rank::Ten => card_games_lib::Rank::Ten,
            Rank::King => card_games_lib::Rank::King,
            Rank::Queen => card_games_lib::Rank::Queen,
            Rank::Jack => card_games_lib::Rank::Jack,
            Rank::Nine => card_games_lib::Rank::Nine,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Piles([Pile<Card>; 3]);

impl Player {
    pub fn next(&self) -> Self {
        match self {
            Player::A => Player::B,
            Player::B => Player::C,
            Player::C => Player::A,
        }
    }

    pub fn from_index(index: usize) -> Option<Player> {
        match index {
            0 => Some(Player::A),
            1 => Some(Player::B),
            2 => Some(Player::C),
            _ => None,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Player::A => 0,
            Player::B => 1,
            Player::C => 2,
        }
    }
}

impl Piles {
    fn empty() -> Piles {
        Piles([pile!(), pile!(), pile!()])
    }

    fn deal(deck: &mut impl Iterator<Item = Card>) -> Piles {
        Piles([
            Pile::deal(deck, 7),
            Pile::deal(deck, 7),
            Pile::deal(deck, 7),
        ])
    }

    fn hand(&self, p: &Player) -> &Pile<Card> {
        &self.0[p.index()]
    }

    fn hand_mut(&mut self, p: &Player) -> &mut Pile<Card> {
        &mut self.0[p.index()]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Fives(usize);

impl Fives {
    pub fn new(f: usize) -> Option<Fives> {
        f.try_into().ok()
    }

    pub fn zero() -> Fives {
        Fives(0)
    }
    pub fn five() -> Fives {
        Fives(1)
    }
    pub fn ten() -> Fives {
        Fives(2)
    }
    pub fn fifteen() -> Fives {
        Fives(3)
    }
    pub fn one_hundred() -> Fives {
        Fives(20)
    }
}

impl From<Fives> for usize {
    fn from(f: Fives) -> usize {
        let Fives(n) = f;
        n * 5
    }
}

impl TryFrom<usize> for Fives {
    type Error = &'static str;

    fn try_from(f: usize) -> Result<Fives, &'static str> {
        if f % 5 == 0 {
            Ok(Fives(f / 5))
        } else {
            Err("Fives must be divisible by five")
        }
    }
}

impl Add<Fives> for Fives {
    type Output = Option<Fives>;

    fn add(self, other: Fives) -> Option<Fives> {
        let Fives(a) = self;
        let Fives(b) = other;
        a.checked_add(b)
            .and_then(|sum| sum.checked_mul(5).map(|_| Fives(sum)))
    }
}

game_states! {
    context: Game,
    states: {
        Bidding {
            hands: Piles,
            prikup: [Card; 3],
            current_bid: (Player, Fives, Player),
        } (bid: Option<Fives>) -> ( AdjustingBid, String ) |this, _context, bid| {
            let (highest_bidder, current_bid, bidding_player) = this.current_bid;
            let next_bidder = bidding_player.next();

            if let Some(bid) = bid {
                let bid = step_try!(bid + current_bid, this, format!("Bid increase too high"));

                StepResult::stay(Bidding {
                    current_bid: (bidding_player, bid, next_bidder),
                    prikup: this.prikup,
                    hands: this.hands,
                })
            } else {
                if next_bidder != highest_bidder {
                    StepResult::stay(Bidding {
                        current_bid: (highest_bidder, current_bid, next_bidder),
                        prikup: this.prikup,
                        hands: this.hands,
                    })
                } else {
                    let [pick_1, pick_2, pick_3] = this.prikup;
                    let mut hands = this.hands;

                    let highest_bidders_hand = hands.hand_mut(&highest_bidder);
                    highest_bidders_hand.add(pick_1);
                    highest_bidders_hand.add(pick_2);
                    highest_bidders_hand.add(pick_3);

                    StepResult::cont(AdjustingBid {
                        bid_winner: highest_bidder,
                        bid: current_bid,
                        hands: hands,
                    })
                }
            }
        },
        AdjustingBid {
            bid_winner: Player,
            bid: Fives,
            hands: Piles
        } (increase: Fives) -> ( Distrubuting , String ) |this, _context, increase| {
            let new_bid = step_try!(this.bid + increase, this, "Bid increase is too high".to_owned());
            StepResult::cont(Distrubuting {
                bid_winner: this.bid_winner,
                hands: this.hands,
                bid: new_bid.into(),
            })
        },
        Distrubuting {
            bid_winner: Player,
            hands: Piles,
            bid: usize
        } (next: card_games_lib::Card, prev: card_games_lib::Card) -> ( Playing, String ) |this, _context, card_for_next, card_for_prev| {
            if !this.hands.hand(&this.bid_winner).contains(&card_for_next)
            || !this.hands.hand(&this.bid_winner).contains(&card_for_prev) {
                return StepResult::fail(this, "Trying to pass a card you don't have".to_owned())
            }
            let mut hands = this.hands;
            let bid_winners_hand = hands.hand_mut(&this.bid_winner);

            let card_for_next = bid_winners_hand.remove(&card_for_next).expect("We checked posession already");
            let card_for_prev = bid_winners_hand.remove(&card_for_prev).expect("We checked posession already");

            let next_player = this.bid_winner.next();
            hands.hand_mut(&next_player).add(card_for_next);

            let next_player = next_player.next();
            hands.hand_mut(&next_player).add(card_for_prev);

            StepResult::cont(Playing {
                bid_winner: this.bid_winner,
                hands: hands,
                trump: None,
                play_area: pile![],
                next_player: this.bid_winner,
                pending_points: [0,0,0],
                taken: Piles::empty(),
                bid: this.bid,
            })
        },
        Playing {
            bid_winner: Player,
            hands: Piles,
            taken: Piles,
            next_player: Player,
            trump: Option<Suit>,
            play_area: Pile<Card>,
            pending_points: [usize; 3],
            bid: usize
        } (player: Player, card: card_games_lib::Card) -> ( Finished, String ) |mut this, _context, player, card| {
            if let Some(initial_card) = this.play_area.get(0) {
                if initial_card.suit() != card.suit() {
                    let players_hand = this.hands.hand(&player);

                    let any_cards_match_suit = players_hand.iter().filter(|c| &&card != c).any(|c| c.suit() == initial_card.suit());

                    if any_cards_match_suit {
                        let message = format!("Cannot play card of {:?} when have {:?} in hand", card.suit(), initial_card.suit());
                        return StepResult::fail(this, message)
                    }
                }
            }

            let played_card = step_try!(this.hands.hand_mut(&player).remove(&card), this, format!("{:?} is not in hand", card));
            let mut play_area = this.play_area;
            let mut next_player = this.next_player.next();
            let mut trump = this.trump;
            let mut pending_points = this.pending_points;

            if play_area.len() == 0 && played_card.rank().is_weddable() {
                let has_marriage = this.hands.hand(&player).iter().any(|c| c.suit() == played_card.suit() && c.rank().is_weddable());
                if has_marriage {
                    trump = Some(played_card.suit().clone());
                    pending_points[player.index()] += played_card.suit().marriage_value();
                }
            }

            play_area.add(played_card);

            if play_area.len() == 3 {
                let lead_suit = play_area.get(0).expect("There is at least 1 card").suit();

                let winning_card = trump.iter()
                                        .flat_map(|trump| play_area.iter().filter(move |c| c.suit() == trump))
                                        .max_by_key(|c|c.rank())
                                        .or_else(|| play_area.iter().filter(move |c| c.suit() == lead_suit).max_by_key(|c|c.rank()))
                                        .expect("There will be a highest card of lead suit");


                let winner = play_area.iter().zip(InfinitePlayerIter(next_player))
                                            .filter(|(card, _)| card == &winning_card)
                                            .map(|(_, player)| player)
                                            .next()
                                            .expect("Some card won");

                next_player = winner;

                this.taken.hand_mut(&winner).extend(play_area.drain());
            }

            StepResult::stay(Playing {
                bid_winner: this.bid_winner,
                hands: this.hands,
                trump: trump,
                pending_points: pending_points,
                next_player: next_player,
                play_area: play_area,
                taken: this.taken,
                bid: this.bid,
            })
        },
        Finished {
            bid_winner: Player,
            taken: Piles,
            pending_points: [usize; 3],
            bid: usize
        } () -> ( Bidding, String ) |_this, _context| {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests;
