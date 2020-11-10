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
    ) -> Option<Error<StateError, State>> {
        self.state.take().and_then(|state| {
            let (state, error) = state.step(&mut self.game, packet);

            let x = state.state();
            dbg!(x);

            self.state = Some(state);
            error
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;
    use tysiac::BiddingA;

    #[test]
    fn it_works() {
        let mut game = Tysiac {
            game: Game,
            state: Some(SomeState::BiddingA(BiddingA::random(&mut thread_rng()))),
        };

        game.feed(0, StateInput::BiddingA(30));
    }
}
