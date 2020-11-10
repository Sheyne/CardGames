use card_games_lib::Error;
use tysiac::{Game, Input, States, StateError, StateName};

pub struct Tysiac {
    game: Game,
    state: Option<States>,
}

impl Tysiac {
    pub fn feed<I>(&mut self, _player: usize, packet: Input) -> Option<Error<StateError, StateName>> {
        self.state.take().and_then(|state| {
            let (state, error) = state.step(&mut self.game, packet);

            self.state = Some(state);
            error
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
