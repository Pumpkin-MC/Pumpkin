pub mod gamemode;
pub mod math;
pub mod permission;
pub mod random;
pub mod text;
pub mod translation;

use std::ops::{Index, IndexMut};

pub use gamemode::GameMode;
pub use permission::PermissionLvl;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Clone)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileAction {
    ForcedNameChange,
    UsingBannedSkin,
}

/// Takes a mutable reference of an index and returns a mutable "slice" where we can mutate both at
/// the same time
pub struct MutableSplitSlice<'a, T> {
    start: &'a mut [T],
    end: &'a mut [T],
}

impl<'a, T> MutableSplitSlice<'a, T> {
    pub fn extract_ith(base: &'a mut [T], index: usize) -> (&'a mut T, Self) {
        let (start, end_inclusive) = base.split_at_mut(index);
        let (value, end) = end_inclusive
            .split_first_mut()
            .expect("Index is not in base slice");

        (value, Self { start, end })
    }

    pub fn len(&self) -> usize {
        self.start.len() + self.end.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl<T> Index<usize> for MutableSplitSlice<'_, T> {
    type Output = T;

    #[allow(clippy::comparison_chain)]
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.start.len() {
            &self.start[index]
        } else if index == self.start.len() {
            panic!("We tried to index into the element that was removed");
        } else {
            &self.end[index - self.start.len() - 1]
        }
    }
}

impl<T> IndexMut<usize> for MutableSplitSlice<'_, T> {
    #[allow(clippy::comparison_chain)]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index < self.start.len() {
            &mut self.start[index]
        } else if index == self.start.len() {
            panic!("We tried to index into the element that was removed");
        } else {
            &mut self.end[index - self.start.len() - 1]
        }
    }
}

#[macro_export]
macro_rules! assert_eq_delta {
    ($x:expr, $y:expr, $d:expr) => {
        if !(2f64 * ($x - $y).abs() <= $d * ($x.abs() + $y.abs())) {
            panic!("{} vs {} ({} vs {})", $x, $y, ($x - $y).abs(), $d);
        }
    };
}
