use tysiac::{Game, Input, InputError, States};

pub struct Tysiac {
    game: Game,
    state: Option<States>,
}

impl Tysiac {
    pub fn feed<I>(&mut self, _player: usize, packet: Input) -> Option<InputError> {
        self.state.take().and_then(|state| {
            let (state, error) = state.step(&mut self.game, packet);

            self.state = Some(state);
            error
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {}
}
