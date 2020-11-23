use super::*;
use rand::thread_rng;
use tysiac::{Bidding, Fives, Player};

#[test]
fn it_works() -> Result<(), Error> {
    let mut game = Tysiac {
        game: Game::default(),
        state: Some(SomeState::Bidding(Bidding::random(&mut thread_rng()))),
    };

    game.feed(1, StateInput::Bidding(None))?;
    game.feed(2, StateInput::Bidding(None))?;
    game.feed(0, StateInput::AdjustingBid(Fives::zero()))?;

    let first_card = match (&(game.state)).as_ref().unwrap() {
        SomeState::Distrubuting(x) => x.hand(&Player::A).iter().next().unwrap().description(),
        _ => panic!("Incorrect state"),
    };

    let second_card = match (&(game.state)).as_ref().unwrap() {
        SomeState::Distrubuting(x) => x
            .hand(&Player::A)
            .iter()
            .skip(1)
            .next()
            .unwrap()
            .description(),
        _ => panic!("Incorrect state"),
    };

    game.feed(0, StateInput::Distrubuting(first_card, second_card))?;

    Ok(())
}
