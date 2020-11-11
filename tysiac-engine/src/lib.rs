use card_games_lib::Error;
use tysiac::{Game, SomeState, State, StateError, StateInput};

pub struct Tysiac {
    game: Game,
    state: Option<SomeState>,
}

impl Tysiac {
    pub fn feed(
        &mut self,
        _player: usize,
        packet: StateInput,
    ) -> Result<(), Error<StateError, State>> {
        if let Some(state) = self.state.take() {
            let (state, error) = state.step(&mut self.game, packet);

            dbg!(state.state());

            self.state = Some(state);
            error
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use tysiac::{Bidding, Fives};

    #[test]
    fn it_works() -> Result<(), Error<StateError, State>> {
        let mut game = Tysiac {
            game: Game,
            state: Some(SomeState::Bidding(Bidding::random(&mut thread_rng()))),
        };

        game.feed(0, StateInput::Bidding(None))?;
        game.feed(0, StateInput::Bidding(None))?;
        game.feed(0, StateInput::AdjustingBid(Fives::zero()))?;

        Ok(())
    }
}
