use std::marker::PhantomData;
#[macro_use]
extern crate typenum;
use std::ops::Add;
use typenum::{op, Integer, Unsigned};

pub struct Ranged<Min, Max, Underlying>(Underlying, PhantomData<Min>, PhantomData<Max>);

macro_rules! define_ranged {
    ($rangeTypes:ident, $conversion:ident, $underlying:ty) => {
        impl<Min, Max> Ranged<Min, Max, $underlying>
        where
            Min: $rangeTypes,
            Max: $rangeTypes,
        {
            pub fn try_new(u: $underlying) -> Option<Self> {
                if u < Min::$conversion {
                    None
                } else if u > Max::$conversion {
                    None
                } else {
                    Some(Ranged(u, PhantomData, PhantomData))
                }
            }

            pub fn wrapping_add(&self, u: $underlying) -> Ranged<Min, Max, $underlying> {
                let sum = self.0 + u;
                let range = Max::$conversion - Min::$conversion + 1;
                Ranged(
                    ((sum - Min::$conversion) % range) + Min::$conversion,
                    PhantomData,
                    PhantomData,
                )
            }
        }
    };
}

define_ranged!(Unsigned, USIZE, usize);
define_ranged!(Integer, ISIZE, isize);

impl<Min, Max, Underlying> Ranged<Min, Max, Underlying>
where
    Underlying: Copy,
{
    pub fn underlying(&self) -> Underlying {
        self.0
    }
}

impl<Min1, Max1, Min2, Max2, Underlying> Add<Ranged<Min2, Max2, Underlying>>
    for Ranged<Min1, Max1, Underlying>
where
    Min1: Integer + Add<Min2>,
    Min2: Integer,
    Max1: Integer + Add<Max2>,
    Max2: Integer,
    Underlying: Add<Underlying, Output = Underlying>,
{
    type Output = Ranged<op!(Min1 + Min2), op!(Max1 + Max2), Underlying>;

    fn add(self, other: Ranged<Min2, Max2, Underlying>) -> Self::Output {
        Ranged(self.0 + other.0, PhantomData, PhantomData)
    }
}

#[macro_export]
macro_rules! define_ranged_enum {
    (@inner_count ; $sum:ty ; $fst:ident, ) => {
        $sum
    };
    (@inner_count ; $sum:ty ; $fst:ident, $($rest:ident,)*) => {
        define_ranged_enum!(@inner_count ; <$sum as Add<::typenum::U1>>::Output ; $($rest,)*)
    };
    (@inner_match ; $idx:expr; $sum:ty ; $($i:ty : $arm:ident)*; ) => {
        match $idx {
            $(<$i>::USIZE => Self::$arm,)*
            _ => unreachable!()
        }
    };
    (@inner_match ; $idx:expr; $sum:ty ; $($i:ty : $arm:ident)*; $fst:ident, $($rest:ident,)*) => {
        define_ranged_enum!(@inner_match ; $idx;
            <$sum as Add<::typenum::U1>>::Output ;
            $($i : $arm)* $sum : $fst;
            $($rest,)*)
    };
    ($name:ident, $($x:ident),+ $(,)?) => {
        #[derive(Debug, PartialEq, Eq)]
        pub enum $name {
            $($x),+
        }

        impl From<$crate::Ranged<::typenum::U0, define_ranged_enum!(@inner_count; ::typenum::U0; $($x,)+), usize>> for $name {
            fn from(x: $crate::Ranged<::typenum::U0, define_ranged_enum!(@inner_count; ::typenum::U0; $($x,)+), usize>)
            -> Self {
                define_ranged_enum!(@inner_match; x.underlying(); ::typenum::U0; ; $($x,)+)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use typenum::*;

    #[test]
    fn can_construct() {
        Ranged::<U2, U4, usize>::try_new(3).unwrap();
    }

    #[test]
    fn holds_constructed() {
        let x = Ranged::<U2, U4, usize>::try_new(3).unwrap();
        assert_eq!(x.underlying(), 3);
    }
    #[test]
    fn cant_under() {
        let x = Ranged::<U2, U4, usize>::try_new(1);
        assert!(x.is_none());
    }
    #[test]
    fn cant_over() {
        let x = Ranged::<U2, U4, usize>::try_new(5);
        assert!(x.is_none());
    }

    #[test]
    fn add_bounds() {
        let x = Ranged::<N2, P4, isize>::try_new(4).unwrap();
        let y = Ranged::<N3, P4, isize>::try_new(4).unwrap();
        let sum: Ranged<N5, P8, isize> = x + y;
        assert_eq!(sum.underlying(), 8);
    }

    #[test]
    fn wrapped_add_no_wrap() {
        let x = Ranged::<N2, P4, isize>::try_new(1).unwrap();
        let y: Ranged<N2, P4, isize> = x.wrapping_add(2);
        assert_eq!(y.underlying(), 3);
    }

    #[test]
    fn wrapped_add_overflow() {
        let x = Ranged::<N2, P4, isize>::try_new(1).unwrap();
        let y: Ranged<N2, P4, isize> = x.wrapping_add(4);
        assert_eq!(y.underlying(), -2);
    }

    define_ranged_enum!(RangedEnumTest, A, B, C);

    #[test]
    fn define_ranged_enum_test() {
        let x = RangedEnumTest::from(Ranged::<U0, U2, usize>::try_new(2).unwrap());
        assert_eq!(x, RangedEnumTest::C);
    }
}
