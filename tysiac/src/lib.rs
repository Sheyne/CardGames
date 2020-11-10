use card_games_lib::{game_states, Step, StepResult};
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
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
        SomeState::BiddingA(BiddingA::random(rng))
    }

    pub fn deal(deck: &mut impl Iterator<Item = Card>) -> Self {
        SomeState::BiddingA(BiddingA::deal(deck))
    }
}

impl BiddingA {
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

impl PartialEq<card_games_lib::Card> for Card {
    fn eq(&self, other: &card_games_lib::Card) -> bool {
        self.description() == *other
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

#[macro_export]
macro_rules! pile {
    () => (
        $crate::Pile(vec![])
    );

    ($($x:expr),+ $(,)?) => (
        Pile(vec![ $($x),+ ])
    );
}

#[derive(Debug, PartialEq, Eq, Default)]
struct Pile(Vec<Card>);

#[derive(Debug, PartialEq, Eq)]
struct Piles([Pile; 3]);

impl Player {
    fn next(&self) -> Self {
        match self {
            Player::A => Player::B,
            Player::B => Player::C,
            Player::C => Player::A,
        }
    }

    fn from_index(index: usize) -> Option<Player> {
        match index {
            0 => Some(Player::A),
            1 => Some(Player::B),
            2 => Some(Player::C),
            _ => None,
        }
    }

    fn index(&self) -> usize {
        match self {
            Player::A => 0,
            Player::B => 1,
            Player::C => 2,
        }
    }
}

impl Extend<Card> for Pile {
    fn extend<T>(&mut self, cards: T)
    where
        T: std::iter::IntoIterator<Item = Card>,
    {
        self.0.extend(cards)
    }
}

impl Pile {
    fn drain(&mut self) -> std::vec::Drain<'_, Card> {
        self.0.drain(..)
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, index: usize) -> Option<&Card> {
        if index < self.0.len() {
            Some(&self.0[index])
        } else {
            None
        }
    }

    fn deal(deck: &mut impl Iterator<Item = Card>, count: usize) -> Pile {
        Pile(deck.take(count).collect())
    }

    fn contains(&self, card: card_games_lib::Card) -> bool {
        self.0.iter().any(|c| c == &card)
    }

    fn remove(&mut self, card: card_games_lib::Card) -> Option<Card> {
        let index = self.0.iter().position(|c| c == &card);

        index.map(|index| self.0.remove(index))
    }

    fn add(&mut self, card: Card) {
        self.0.push(card)
    }

    fn iter(&self) -> impl Iterator<Item = &Card> {
        self.0.iter()
    }
}

impl Piles {
    fn empty() -> Piles {
        Piles([Pile::default(), Pile::default(), Pile::default()])
    }

    fn deal(deck: &mut impl Iterator<Item = Card>) -> Piles {
        Piles([
            Pile::deal(deck, 7),
            Pile::deal(deck, 7),
            Pile::deal(deck, 7),
        ])
    }

    fn hand(&self, p: &Player) -> &Pile {
        &self.0[p.index()]
    }

    fn hand_mut(&mut self, p: &Player) -> &mut Pile {
        &mut self.0[p.index()]
    }
}

game_states! {
    context: Game,
    states: {
        BiddingA {
            hands: Piles,
            prikup: [Card; 3]
        } (bid: usize) -> ( BiddingB, String ) |this, _context, bid| {
            StepResult::cont(BiddingB {
                hands: this.hands,
                prikup: this.prikup,
                bids: [bid],
            })
        },
        BiddingB {
            hands: Piles,
            prikup: [Card; 3],
            bids: [usize; 1]
        } (bid: usize) -> ( BiddingC, String ) |this, _context, bid| {
            StepResult::cont(BiddingC {
                hands: this.hands,
                prikup: this.prikup,
                bids: [this.bids[0], bid],
            })
        },
        BiddingC {
            hands: Piles,
            prikup: [Card; 3],
            bids: [usize; 2]
        } (bid: usize) -> ( AdjustingBid , String ) |this, _context, bid| {
            let bids = [this.bids[0], this.bids[1], bid];

            let (highest_bidder, winning_bid) = bids
                .iter()
                .enumerate()
                .max_by_key(|(_, val)| *val)
                .expect("We know there's at least one element");
            let highest_bidder = Player::from_index(highest_bidder).expect("Index is valid by construction");

            let mut hands = this.hands;

            let [pick_1, pick_2, pick_3] = this.prikup;

            let highest_bidders_hand = hands.hand_mut(&highest_bidder);
            highest_bidders_hand.add(pick_1);
            highest_bidders_hand.add(pick_2);
            highest_bidders_hand.add(pick_3);

            StepResult::cont(AdjustingBid {
                bid_winner: highest_bidder,
                bid: *winning_bid,
                hands: hands,
            })
        },
        AdjustingBid {
            bid_winner: Player,
            bid: usize,
            hands: Piles
        } (increase: usize) -> ( Distrubuting , String ) |this, _context, increase| {
            if let Some(new_bid) = this.bid.checked_add(increase) {
                StepResult::cont(Distrubuting {
                    bid_winner: this.bid_winner,
                    hands: this.hands,
                    bid: new_bid,
                })
            } else {
                StepResult::fail(this, "Bid increase is too high".to_owned())
            }
        },
        Distrubuting {
            bid_winner: Player,
            hands: Piles,
            bid: usize
        } (next: card_games_lib::Card, prev: card_games_lib::Card) -> ( Playing, String ) |this, _context, card_for_next, card_for_prev| {
            if !this.hands.hand(&this.bid_winner).contains(card_for_next)
            || !this.hands.hand(&this.bid_winner).contains(card_for_prev) {
                return StepResult::fail(this, "Trying to pass a card you don't have".to_owned())
            }
            let mut hands = this.hands;
            let bid_winners_hand = hands.hand_mut(&this.bid_winner);

            let card_for_next = bid_winners_hand.remove(card_for_next).expect("We checked posession already");
            let card_for_prev = bid_winners_hand.remove(card_for_prev).expect("We checked posession already");

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
            play_area: Pile,
            pending_points: [usize; 3],
            bid: usize
        } (player: Player, card: card_games_lib::Card) -> ( Finished, String ) |mut this, _context, player, card| {
            if let Some(initial_card) = this.play_area.get(0) {
                if initial_card.suit() != card.suit() {
                    let players_hand = this.hands.hand(&player);

                    let any_cards_match_suit = players_hand.iter().filter(|c| c != &&card).any(|c| c.suit() == initial_card.suit());

                    if any_cards_match_suit {
                        let message = format!("Cannot play card of {:?} when have {:?} in hand", card.suit(), initial_card.suit());
                        return StepResult::fail(this, message)
                    }
                }
            }

            if let Some(played_card) = this.hands.hand_mut(&player).remove(card) {
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
            } else {
                StepResult::fail(this, format!("{:?} is not in hand", card))
            }
        },
        Finished {
            bid_winner: Player,
            taken: Piles,
            pending_points: [usize; 3],
            bid: usize
        } () -> ( BiddingA, String ) |_this, _context| {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use card_games_lib::{Error, Step};
    use {Rank::*, Suit::*};

    fn test_prikup_1() -> [Card; 3] {
        [Card(Queen, Clubs), Card(Jack, Clubs), Card(Nine, Clubs)]
    }

    fn test_hands_1() -> Piles {
        Piles([
            pile![
                Card(Ace, Hearts),
                Card(Ten, Hearts),
                Card(King, Hearts),
                Card(Queen, Hearts),
                Card(Jack, Hearts),
                Card(Nine, Hearts),
                Card(Ace, Clubs),
            ],
            pile![
                Card(Ace, Diamonds),
                Card(Ten, Diamonds),
                Card(King, Diamonds),
                Card(Queen, Diamonds),
                Card(Jack, Diamonds),
                Card(Nine, Diamonds),
                Card(Ten, Clubs),
            ],
            pile![
                Card(Ace, Spades),
                Card(Ten, Spades),
                Card(King, Spades),
                Card(Queen, Spades),
                Card(Jack, Spades),
                Card(Nine, Spades),
                Card(King, Clubs),
            ],
        ])
    }

    #[test]
    fn bid_a() -> Result<(), Error<String, State>> {
        let mut game = Game;
        let state = BiddingA {
            hands: test_hands_1(),
            prikup: test_prikup_1(),
        };

        let state: BiddingB = state.step(&mut game, 10).next()?;

        assert_eq!(state.bids[0], 10);

        let state: BiddingC = state.step(&mut game, 20).next()?;

        assert_eq!(state.bids[1], 20);

        let state: AdjustingBid = state.step(&mut game, 30).next()?;

        assert_eq!(state.bid, 30);
        assert_eq!(state.bid_winner, Player::C);
        assert_eq!(
            state.hands.hand(&Player::C),
            &Pile(vec![
                Card(Ace, Spades),
                Card(Ten, Spades),
                Card(King, Spades),
                Card(Queen, Spades),
                Card(Jack, Spades),
                Card(Nine, Spades),
                Card(King, Clubs),
                Card(Queen, Clubs),
                Card(Jack, Clubs),
                Card(Nine, Clubs),
            ])
        );
        assert_eq!(state.hands.hand(&Player::A).iter().count(), 7);
        assert_eq!(state.hands.hand(&Player::B).iter().count(), 7);

        let state: Distrubuting = state.step(&mut game, 10).next()?;

        assert_eq!(state.bid, 40);
        assert_eq!(state.bid_winner, Player::C);
        let state: Playing = state
            .step(&mut game, {
                use card_games_lib::{Card, Rank::*, Suit::*};
                (Card(Jack, Spades), Card(King, Spades))
            })
            .next()?;

        assert_eq!(
            state.hands.hand(&Player::C),
            &pile![
                Card(Ace, Spades),
                Card(Ten, Spades),
                Card(Queen, Spades),
                Card(Nine, Spades),
                Card(King, Clubs),
                Card(Queen, Clubs),
                Card(Jack, Clubs),
                Card(Nine, Clubs)
            ]
        );
        assert_eq!(
            state.hands.hand(&Player::A),
            &pile![
                Card(Ace, Hearts),
                Card(Ten, Hearts),
                Card(King, Hearts),
                Card(Queen, Hearts),
                Card(Jack, Hearts),
                Card(Nine, Hearts),
                Card(Ace, Clubs),
                Card(Jack, Spades),
            ]
        );
        assert_eq!(
            state.hands.hand(&Player::B),
            &pile![
                Card(Ace, Diamonds),
                Card(Ten, Diamonds),
                Card(King, Diamonds),
                Card(Queen, Diamonds),
                Card(Jack, Diamonds),
                Card(Nine, Diamonds),
                Card(Ten, Clubs),
                Card(King, Spades),
            ]
        );

        Ok(())
    }
}
