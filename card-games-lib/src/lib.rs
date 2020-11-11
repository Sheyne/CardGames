mod states;
pub use states::*;

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

#[macro_export]
macro_rules! pile {
    () => (
        $crate::Pile::default()
    );

    ($($x:expr),+ $(,)?) => (
        Pile::from_vec(vec![ $($x),+ ])
    );
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
