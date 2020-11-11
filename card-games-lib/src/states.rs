use either::Either;

pub trait State {
    type Id;

    fn state() -> Self::Id;
}

pub trait Step {
    type SomeState;
    type Context;
    type Input;
    type ThisState;
    type NextState: Into<Self::SomeState>;
    type Error;

    fn step(
        self,
        game: &mut Self::Context,
        input: Self::Input,
    ) -> StepResult<Self::ThisState, Self::NextState, Self::Error>;
}

#[derive(Debug)]
pub struct StepResult<ThisState, NextState, Error>(
    pub Either<ThisState, NextState>,
    pub Result<(), Error>,
);

#[derive(Debug, PartialEq, Eq)]
pub enum Error<StepError, State> {
    StepError(StepError),
    NotInCorrectStateError { held: State, given: State },
}

impl<ThisState, NextState, Error> From<StepResult<ThisState, NextState, Error>>
    for Result<Either<ThisState, NextState>, Error>
{
    fn from(src: StepResult<ThisState, NextState, Error>) -> Self {
        let StepResult(next, error) = src;
        error.map(|_| next)
    }
}

impl<ThisState, NextState, E, N> StepResult<ThisState, NextState, E>
where
    ThisState: State<Id = N>,
    NextState: State<Id = N>,
{
    pub fn this(self) -> Result<ThisState, Error<E, N>> {
        let Self(next, error) = self;
        error.map_err(|e| Error::StepError(e)).and_then(|_| {
            next.left().ok_or(Error::NotInCorrectStateError {
                held: ThisState::state(),
                given: NextState::state(),
            })
        })
    }

    pub fn next(self) -> Result<NextState, Error<E, N>> {
        let Self(next, error) = self;
        error.map_err(|e| Error::StepError(e)).and_then(|_| {
            next.right().ok_or(Error::NotInCorrectStateError {
                held: NextState::state(),
                given: ThisState::state(),
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
macro_rules! step_try {
    ($x:expr, $this:ident, $fail:expr) => {
        if let Some(x) = $x {
            x
        } else {
            let arg = $fail;
            return StepResult::fail($this, arg);
        }
    };
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
            #[derive(Debug)]
            pub struct $state {
                $( $field : $type ), *
            }

            impl $crate::State for $state {
                type Id = self::State;

                fn state() -> Self::Id {
                    return Self::Id::$state;
                }
            }

            impl From<$state> for self::SomeState {
                fn from(state: $state) -> self::SomeState {
                    self::SomeState::$state(state)
                }
            }

            impl $crate::Step for $state {
                type SomeState = SomeState;
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

        #[derive(Debug)]
        pub enum StateInput {
            $($state ( $( $arg_type),* ) ), +
        }

        impl self::StateInput {
            pub fn state(&self) -> self::State {
                match self {
                    $(self::StateInput::$state($( $arg ),*) => self::State::$state ), +
                }
            }
        }

        #[derive(Debug, PartialEq, Eq)]
        pub enum StateError {
            $( $state($error) ), +
        }

        impl self::StateError {
            pub fn state(&self) -> self::State {
                match self {
                    $(self::StateError::$state(_) => self::State::$state ), +
                }
            }
        }

        #[derive(Debug)]
        pub enum SomeState {
            $($state($state)), +
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum State {
            $($state), +
        }

        impl SomeState {
            pub fn state(&self) -> self::State {
                match self {
                    $(self::SomeState::$state(_) => self::State::$state ), +
                }
            }

            pub fn step(self, context: &mut $context, input: self::StateInput) -> (Self, ::core::result::Result<(), $crate::Error<self::StateError, self::State>>)
            {
                match (self, input) {
                    $((self::SomeState::$state(state), self::StateInput::$state( $( $arg ), * )) => {
                        let self::StepResult(next, result) = state.step(context, ($( $arg ), *));
                        let err = result.map_err(|e| $crate::Error::StepError(self::StateError::$state(e)));
                        (::core::convert::Into::into(next), err)
                    }),+
                    $((SomeState::$state(a), input) =>{
                        (self::SomeState::$state(a), Err($crate::Error::NotInCorrectStateError{
                            held: self::State::$state,
                            given: input.state()
                        }))
                    }),+
                }
            }
        }

        impl<A, B> From<::either::Either<A, B>> for SomeState
        where A : Into<SomeState>,
            B : Into<SomeState>
        {
            fn from(s: ::either::Either<A, B>) -> SomeState {
                match s {
                    ::either::Either::Left(a) => a.into(),
                    ::either::Either::Right(b) => b.into()
                }
            }
        }
    };
}
