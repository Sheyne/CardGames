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
pub enum Error<StepError> {
    StepError(StepError),
    NotInCorrectStateError,
}

impl<ThisState, NextState, Error> From<StepResult<ThisState, NextState, Error>>
    for Result<Either<ThisState, NextState>, Error>
{
    fn from(src: StepResult<ThisState, NextState, Error>) -> Self {
        let StepResult(next, error) = src;
        error.map(|_| next)
    }
}

impl<ThisState, NextState, E> StepResult<ThisState, NextState, E> {
    pub fn this(self) -> Result<ThisState, Error<E>> {
        let Self(next, error) = self;
        error
            .map_err(|e| Error::StepError(e))
            .and_then(|_| next.left().ok_or(Error::NotInCorrectStateError))
    }

    pub fn next(self) -> Result<NextState, Error<E>> {
        let Self(next, error) = self;
        error
            .map_err(|e| Error::StepError(e))
            .and_then(|_| next.right().ok_or(Error::NotInCorrectStateError))
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
    { $( $state:ident
         { $( $field:ident : $type:ty),* }
         ($( $arg:ident : $arg_type:ty),*) -> ( $next_state:ty , $error:ty )
         $body:expr
        ),+ } => {
        $(
            pub struct $state {
                $( $field : $type ), *
            }

            impl From<$state> for self::States {
                fn from(state: $state) -> self::States {
                    self::States::$state(state)
                }
            }

            impl<ThisState, Error> ::core::convert::TryFrom<$crate::StepResult<ThisState, $state, Error>> for $state {
                type Error = $crate::Error<Error>;
                fn try_from(result: $crate::StepResult<ThisState, $state, Error>) -> ::core::result::Result<Self, $crate::Error<Error>> {
                    result.next()
                }
            }

            impl $crate::Step for $state {
                type States = States;
                type Context = Game;
                type Input = ($( $arg_type ), * );
                type ThisState = self::$state;
                type NextState = $next_state;
                type Error = $error;

                fn step(self, context: &mut Game, input: Self::Input) -> $crate::StepResult<Self::ThisState, Self::NextState, Self::Error>
                {
                    let ($( $arg ), * ) = input;
                    let func: &Fn(Self, &mut Game, $( $arg_type ), *) -> $crate::StepResult<Self::ThisState, Self::NextState, Self::Error> = &$body;
                    func(self, context, $( $arg ), * )
                }
            }
        )+


        pub struct InputError(self::StateName, self::Input);

        pub enum Input {
            $($state ( $( $arg_type),* ) ), +
        }

        pub enum States {
            $($state($state)), +
        }

        pub enum StateName {
            $($state), +
        }

        impl States {
            pub fn step(self, context: &mut self::Game, input: self::Input) -> (Self, ::core::option::Option<self::InputError>)
            {
                match (self, input) {
                    $((self::States::$state(state), self::Input::$state( $( $arg ), * )) => {
                        let self::StepResult(next, result) = state.step(context, ($( $arg ), *));
                        (::core::convert::Into::into(next), None)
                    }),+
                    $((States::$state(a), input) =>{
                        (self::States::$state(a), Some(self::InputError(self::StateName::$state, input)))
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
