use tysiac::{Game, Player, SomeState, State, StateError, StateInput};

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    Game(card_games_lib::Error<StateError, State>),
    NoState,
    IncorrectPlayer {
        current: Player,
        attempted: Option<Player>,
    },
}

pub struct Tysiac {
    game: Game,
    state: Option<SomeState>,
}

impl Tysiac {
    pub fn feed(&mut self, player: usize, packet: StateInput) -> Result<(), Error> {
        if let Some(state) = self.state.take() {
            let next_player = state.next_player();
            if next_player.index() != player {
                self.state = Some(state);

                return Err(Error::IncorrectPlayer {
                    current: next_player,
                    attempted: Player::from_index(player),
                });
            }

            let (state, error) = state.step(&mut self.game, packet);

            dbg!(state.state());

            self.state = Some(state);
            error.map_err(|x| Error::Game(x))
        } else {
            Err(Error::NoState)
        }
    }
}

#[cfg(test)]
mod tests;
