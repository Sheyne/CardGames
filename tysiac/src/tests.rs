use super::*;
use card_games_lib::{pile, Error, Step};
use std::convert::TryInto;
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
    let mut game = Game::default();
    let state = Bidding {
        current_bid: (Player::A, Fives::one_hundred(), Player::B),
        hands: test_hands_1(),
        prikup: test_prikup_1(),
    };

    let state: Bidding = state.step(&mut game, None).this()?;
    assert_eq!(
        state.current_bid,
        (Player::A, Fives::one_hundred(), Player::C)
    );

    let state: Bidding = state.step(&mut game, Some(30.try_into().unwrap())).this()?;
    assert_eq!(
        state.current_bid,
        (Player::C, 130.try_into().unwrap(), Player::A)
    );

    let state: Bidding = state.step(&mut game, None).this()?;
    assert_eq!(
        state.current_bid,
        (Player::C, 130.try_into().unwrap(), Player::B)
    );

    let state: AdjustingBid = state.step(&mut game, None).next()?;

    assert_eq!(state.bid, 130.try_into().unwrap());
    assert_eq!(state.bid_winner, Player::C);
    assert_eq!(
        state.hands.hand(&Player::C),
        &pile![
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
    assert_eq!(state.hands.hand(&Player::A).iter().count(), 7);
    assert_eq!(state.hands.hand(&Player::B).iter().count(), 7);

    let state: Distrubuting = state.step(&mut game, Fives::ten()).next()?;

    assert_eq!(state.bid, 140);
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
