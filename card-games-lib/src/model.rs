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

#[derive(Debug, PartialEq, Eq)]
pub struct Pile<T>(Vec<T>);

impl Card {
    pub fn rank(self) -> Rank {
        self.0
    }
    pub fn suit(self) -> Suit {
        self.1
    }
}

impl<T> Default for Pile<T> {
    fn default() -> Pile<T> {
        Pile(vec![])
    }
}

impl<T> Extend<T> for Pile<T> {
    fn extend<U>(&mut self, cards: U)
    where
        U: std::iter::IntoIterator<Item = T>,
    {
        self.0.extend(cards)
    }
}

impl<T> Pile<T> {
    pub fn from_vec(v: Vec<T>) -> Pile<T> {
        Pile(v)
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, T> {
        self.0.drain(..)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extract<D>(&mut self, descriptions: impl Iterator<Item = D>) -> Option<Vec<T>>
    where
        D: PartialEq<T>,
    {
        let mut taken = std::collections::BTreeSet::new();

        for desc in descriptions {
            if let Some(idx) = self
                .0
                .iter()
                .enumerate()
                .filter(|(idx, c)| !taken.contains(idx) && desc == **c)
                .map(|(idx, _)| idx)
                .next()
            {
                taken.insert(idx);
            } else {
                return None;
            }
        }

        Some(taken.iter().rev().map(|idx| self.0.remove(*idx)).collect())
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.0.len() {
            Some(&self.0[index])
        } else {
            None
        }
    }

    pub fn deal(deck: &mut impl Iterator<Item = T>, count: usize) -> Pile<T> {
        Pile(deck.take(count).collect())
    }

    pub fn contains<D>(&self, desc: &D) -> bool
    where
        D: PartialEq<T>,
    {
        self.0.iter().any(|c| desc == c)
    }

    pub fn remove<D>(&mut self, desc: &D) -> Option<T>
    where
        D: PartialEq<T>,
    {
        let index = self.0.iter().position(|c| desc == c);

        index.map(|index| self.0.remove(index))
    }

    pub fn add(&mut self, card: T) {
        self.0.push(card)
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

#[macro_export]
macro_rules! pile_extract {
    (@vec_to_tuple_xs $vec:ident ; $($accum:expr),* , ; ) => {
        ($($accum),*)
    };
    (@vec_to_tuple_xs $vec:ident ; $($accum:expr,)* ; $fst:expr, $($rest:expr,)* ) => {
        pile_extract!(@vec_to_tuple_xs $vec ; $($accum,)* $vec.remove(0), ; $($rest,)* )
    };


    ($pile:expr, $($x:expr),+ $(,)?) => {
        {
            let pile = $pile;

            let extracted = pile.extract(vec![$($x),+].into_iter());

            if let Some(mut extracted) = extracted {
                Some((
                    pile_extract!(@vec_to_tuple_xs extracted ; ; $($x,)+)
                ))
            } else {
                None
            }
        }
    }
}

#[macro_export]
macro_rules! pile {
    () => (
        $crate::Pile::default()
    );

    ($($x:expr),+ $(,)?) => (
        Pile::from_vec(vec![ $($x),+ ])
    );
}
