use card_games_lib::{game_states, Step, StepResult};
use either::Either;
use itertools::Itertools;
use rand::{seq::SliceRandom, Rng};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

pub struct Game;

#[derive(EnumIter)]
pub enum Player {
    A,
    B,
    C,
}

impl States {
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        States::BiddingA(BiddingA::random(rng))
    }

    fn deal(deck: &mut impl Iterator<Item = Card>) -> Self {
        States::BiddingA(BiddingA::deal(deck))
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

    fn deal(deck: &mut impl Iterator<Item = Card>) -> Self {
        Self {
            hands: Hands::deal(deck),
            prikup: [
                deck.next().unwrap(),
                deck.next().unwrap(),
                deck.next().unwrap(),
            ],
        }
    }

    fn new(
        a: Vec<Card>,
        b: Vec<Card>,
        c: Vec<Card>,
        mut prikup: Vec<Card>,
    ) -> Result<Self, String> {
        if a.len() != 7 {
            return Err("Hand a is not the right length".to_owned());
        }

        if b.len() != 7 {
            return Err("Hand b is not the right length".to_owned());
        }

        if c.len() != 7 {
            return Err("Hand c is not the right length".to_owned());
        }

        if prikup.len() != 3 {
            return Err("Prikup is not the right length".to_owned());
        }

        Ok(Self {
            hands: Hands::new(a, b, c),
            prikup: [prikup.remove(0), prikup.remove(0), prikup.remove(0)],
        })
    }
}

#[derive(EnumIter, Clone, Debug, PartialEq, Eq)]
pub enum Suit {
    Spades,
    Clubs,
    Diamonds,
    Hearts,
}

#[derive(EnumIter, Clone, Debug, PartialEq, Eq)]
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
pub struct Card(Rank, Suit);

impl Card {
    pub fn suit(self) -> Suit {
        self.1
    }

    pub fn rank(self) -> Rank {
        self.0
    }
}

struct Hands([Vec<Card>; 3]);

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

impl Hands {
    fn empty() -> Hands {
        Hands([vec![], vec![], vec![]])
    }

    fn new(a: Vec<Card>, b: Vec<Card>, c: Vec<Card>) -> Self {
        Self([a, b, c])
    }

    fn deal_hand(deck: &mut impl Iterator<Item = Card>) -> Vec<Card> {
        deck.take(7).collect()
    }

    fn deal(deck: &mut impl Iterator<Item = Card>) -> Hands {
        Hands([
            Self::deal_hand(deck),
            Self::deal_hand(deck),
            Self::deal_hand(deck),
        ])
    }

    fn hand(&self, p: &Player) -> &Vec<Card> {
        &self.0[p.index()]
    }

    fn hand_mut(&mut self, p: &Player) -> &mut Vec<Card> {
        &mut self.0[p.index()]
    }
}

game_states! {
    BiddingA {
        hands: Hands,
        prikup: [Card; 3]
    } (bid: usize) -> ( BiddingB, (), String ) |this, _context, bid| {
        StepResult::cont(BiddingB {
            hands: this.hands,
            prikup: this.prikup,
            bids: [bid],
        })
    },
    BiddingB {
        hands: Hands,
        prikup: [Card; 3],
        bids: [usize; 1]
    } (bid: usize) -> ( BiddingC, (), String ) |this, _context, bid| {
        StepResult::cont(BiddingC {
            hands: this.hands,
            prikup: this.prikup,
            bids: [this.bids[0], bid],
        })
    },
    BiddingC {
        hands: Hands,
        prikup: [Card; 3],
        bids: [usize; 2]
    } (bid: usize) -> ( AdjustingBid , (), String ) |this, _context, bid| {
        let bids = [this.bids[0], this.bids[1], bid];

        let highest_bidder = bids
            .iter()
            .enumerate()
            .max_by_key(|(_, val)| *val)
            .and_then(|(idx, _)| Player::from_index(idx))
            .expect("We know there's at least one element and its at a valid index");

        let mut hands = this.hands;

        let [pick_1, pick_2, pick_3] = this.prikup;

        let highest_bidders_hand = hands.hand_mut(&highest_bidder);
        highest_bidders_hand.push(pick_1);
        highest_bidders_hand.push(pick_2);
        highest_bidders_hand.push(pick_3);

        StepResult::cont(AdjustingBid {
            bid_winner: highest_bidder,
            hands: hands,
            bids: bids,
        })
    },
    AdjustingBid {
        bid_winner: Player,
        hands: Hands,
        bids: [usize; 3]
    } (increase: usize) -> ( Distrubuting , (), String ) |this, _context, increase| {
        let winning_bid = this.bids[this.bid_winner.index()];

        if let Some(new_bid) = winning_bid.checked_add(increase) {
            let mut bids = this.bids;
            bids[this.bid_winner.index()] = new_bid;

            StepResult::cont(Distrubuting {
                bid_winner: this.bid_winner,
                hands: this.hands,
                bids: bids,
            })
        } else {
            StepResult::fail(this, "Bid increase is too high".to_owned())
        }
    },
    Distrubuting {
        bid_winner: Player,
        hands: Hands,
        bids: [usize; 3]
    } (next: usize, prev: usize) -> ( Playing, (), String ) |this, _context, index_for_b, index_for_c| {
        let bid_winner_hand_size = this.hands.hand(&this.bid_winner).len();
        if index_for_b >= bid_winner_hand_size || index_for_c >= bid_winner_hand_size {
            return StepResult::fail(this, "Trying to pass a card at a non-existant index".to_owned())
        }
        let mut hands = this.hands;
        let bid_winners_hand = hands.hand_mut(&this.bid_winner);

        let (card_for_b, card_for_c) = if index_for_b > index_for_c {
            (
                bid_winners_hand.remove(index_for_b),
                bid_winners_hand.remove(index_for_c),
            )
        } else {
            let (c, b) = (
                bid_winners_hand.remove(index_for_c),
                bid_winners_hand.remove(index_for_b),
            );
            (b, c)
        };

        let next_player = this.bid_winner.next();
        hands.hand_mut(&next_player).push(card_for_b);

        let next_player = next_player.next();
        hands.hand_mut(&next_player).push(card_for_c);

        StepResult::cont(Playing {
            bid_winner: this.bid_winner,
            hands: hands,
            taken: Hands::empty(),
            bids: this.bids,
        })
    },
    Playing {
        bid_winner: Player,
        hands: Hands,
        taken: Hands,
        bids: [usize; 3]
    } (player: Player, card: usize) -> ( Either<Playing, Finished>, (), String ) |_this, _context, _player, _card| {
        todo!()
    },
    Finished {
        bid_winner: Player,
        taken: Hands,
        bids: [usize; 3]
    } () -> ( BiddingA, (), String ) |_this, _context| {
        todo!()
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

    fn test_hands_1() -> Hands {
        Hands::new(
            vec![
                Card(Ace, Hearts),
                Card(Ten, Hearts),
                Card(King, Hearts),
                Card(Queen, Hearts),
                Card(Jack, Hearts),
                Card(Nine, Hearts),
                Card(Ace, Clubs),
            ],
            vec![
                Card(Ace, Diamonds),
                Card(Ten, Diamonds),
                Card(King, Diamonds),
                Card(Queen, Diamonds),
                Card(Jack, Diamonds),
                Card(Nine, Diamonds),
                Card(Ten, Clubs),
            ],
            vec![
                Card(Ace, Spades),
                Card(Ten, Spades),
                Card(King, Spades),
                Card(Queen, Spades),
                Card(Jack, Spades),
                Card(Nine, Spades),
                Card(King, Clubs),
            ],
        )
    }

    #[test]
    fn bid_a() -> Result<(), Error<String>> {
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

        assert_eq!(state.bids, [10, 20, 30]);
        assert_eq!(
            state.hands.hand(&Player::C),
            &vec![
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
            ]
        );
        assert_eq!(state.hands.hand(&Player::A).len(), 7);
        assert_eq!(state.hands.hand(&Player::B).len(), 7);

        let state: Distrubuting = state.step(&mut game, 10).next()?;

        assert_eq!(state.bids, [10, 20, 40]);

        let state: Playing = state.step(&mut game, (4, 2)).next()?;

        assert_eq!(
            state.hands.hand(&Player::C),
            &vec![
                Card(Ace, Spades),
                Card(Ten, Spades),
                Card(Queen, Spades),
                Card(Nine, Spades),
                Card(King, Clubs),
                Card(Queen, Clubs),
                Card(Jack, Clubs),
                Card(Nine, Clubs),
            ]
        );
        assert_eq!(
            state.hands.hand(&Player::A),
            &vec![
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
            &vec![
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
