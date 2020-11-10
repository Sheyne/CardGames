use either::Either;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Suit {
    Diamonds,
    Clubs,
    Hearts,
    Spades,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Card(pub Rank, pub Suit);

impl Card {
    pub fn rank(self) -> Rank {
        self.0
    }
    pub fn suit(self) -> Suit {
        self.1
    }
}

pub trait Step {
    type States;
    type Context;
    type Input;
    type ThisState;
    type NextState: Into<Self::States>;
    type Error;

    fn step(
        self,
        game: &mut Self::Context,
        input: Self::Input,
    ) -> StepResult<Self::ThisState, Self::NextState, Self::Error>;
}

pub struct StepResult<ThisState, NextState, Error>(
    pub Either<ThisState, NextState>,
    pub Result<(), Error>,
);

#[derive(Debug)]
pub enum Error<StepError, StateName> {
    StepError(StepError),
    NotInCorrectStateError {
        expected: StateName,
        given: StateName,
    },
}

impl<ThisState, NextState, Error> From<StepResult<ThisState, NextState, Error>>
    for Result<Either<ThisState, NextState>, Error>
{
    fn from(src: StepResult<ThisState, NextState, Error>) -> Self {
        let StepResult(next, error) = src;
        error.map(|_| next)
    }
}

pub trait State {
    type Name;

    fn name() -> Self::Name;
}

impl<ThisState, NextState, E, N> StepResult<ThisState, NextState, E>
where
    ThisState: State<Name = N>,
    NextState: State<Name = N>,
{
    pub fn this(self) -> Result<ThisState, Error<E, N>> {
        let Self(next, error) = self;
        error.map_err(|e| Error::StepError(e)).and_then(|_| {
            next.left().ok_or(Error::NotInCorrectStateError {
                expected: ThisState::name(),
                given: NextState::name(),
            })
        })
    }

    pub fn next(self) -> Result<NextState, Error<E, N>> {
        let Self(next, error) = self;
        error.map_err(|e| Error::StepError(e)).and_then(|_| {
            next.right().ok_or(Error::NotInCorrectStateError {
                expected: NextState::name(),
                given: ThisState::name(),
            })
        })
    }

    pub fn stay(this: ThisState) -> Self {
        Self(Either::Left(this), Ok(()))
    }

    pub fn cont(next: NextState) -> Self {
        Self(Either::Right(next), Ok(()))
    }

    pub fn fail(this: ThisState, result: E) -> Self {
        Self(Either::Left(this), Err(result))
    }

    pub fn fail_continue(next: NextState, result: E) -> Self {
        Self(Either::Right(next), Err(result))
    }
}

#[macro_export]
macro_rules! game_states {
    { context: $context:ty,
      states: {
        $( $state:ident
         { $( $field:ident : $type:ty),* $(,)? }
         ( $( $arg:ident : $arg_type:ty),* $(,)? ) -> ( $next_state:ty , $error:ty $(,)? )
         $body:expr
        ),+ $(,)? }
     } => {
        $(
            pub struct $state {
                $( $field : $type ), *
            }

            impl $crate::State for $state {
                type Name = self::StateName;

                fn name() -> self::StateName {
                    return self::StateName::$state;
                }
            }

            impl From<$state> for self::States {
                fn from(state: $state) -> self::States {
                    self::States::$state(state)
                }
            }

            impl $crate::Step for $state {
                type States = States;
                type Context = $context;
                type Input = ($( $arg_type ), * );
                type ThisState = self::$state;
                type NextState = $next_state;
                type Error = $error;

                fn step(self, context: &mut $context, input: Self::Input) -> $crate::StepResult<Self::ThisState, Self::NextState, Self::Error>
                {
                    let ($( $arg ), * ) = input;
                    let func: &Fn(Self, &mut $context, $( $arg_type ), *) -> $crate::StepResult<Self::ThisState, Self::NextState, Self::Error> = &$body;
                    func(self, context, $( $arg ), * )
                }
            }
        )+

        pub enum Input {
            $($state ( $( $arg_type),* ) ), +
        }

        pub enum StateError {
            $( $state($error) ), +
        }

        pub enum States {
            $($state($state)), +
        }

        pub enum StateName {
            $($state), +
        }

        impl States {
            pub fn step(self, context: &mut $context, input: self::Input) -> (Self, ::core::option::Option<$crate::Error<self::StateError, self::StateName>>)
            {
                match (self, input) {
                    $((self::States::$state(state), self::Input::$state( $( $arg ), * )) => {
                        let self::StepResult(next, result) = state.step(context, ($( $arg ), *));
                        (::core::convert::Into::into(next), todo!())
                    }),+
                    $((States::$state(a), input) =>{
                        (self::States::$state(a), Some($crate::Error::NotInCorrectStateError{
                            expected: self::StateName::$state, 
                            given: todo!()
                        }))
                    }),+
                }
            }
        }

        impl<A, B> From<::either::Either<A, B>> for States
        where A : Into<States>,
            B : Into<States>
        {
            fn from(s: ::either::Either<A, B>) -> States {
                match s {
                    ::either::Either::Left(a) => a.into(),
                    ::either::Either::Right(b) => b.into()
                }
            }
        }
    };
}
