use std::sync::{Mutex, RwLock, RwLockWriteGuard};

use bytes::Bytes;
use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
pub use pumpkin_protocol::codec::recipe::DynamicRecipe;

pub struct RecipeManager {
    dynamic_recipes: RwLock<Vec<DynamicRecipe>>,
    /// The packet with the whole recipe book. Dropped when the recipes change.
    book_packet: Mutex<Option<Bytes>>,
}

impl Default for RecipeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeManager {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            dynamic_recipes: RwLock::new(Vec::new()),
            book_packet: Mutex::new(None),
        }
    }

    /// The recipes to change. The packet built from them is dropped while they are locked,
    /// so one built from the old ones is not kept.
    fn change(&self) -> RwLockWriteGuard<'_, Vec<DynamicRecipe>> {
        let recipes = self
            .dynamic_recipes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.book_packet
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        recipes
    }

    /// The packet that gives a client the whole recipe book: `build` makes it from the
    /// dynamic recipes. Every client gets the same one, so it is built once until the
    /// recipes change.
    pub fn book_packet(
        &self,
        build: impl FnOnce(&[DynamicRecipe]) -> Option<Bytes>,
    ) -> Option<Bytes> {
        let recipes = self
            .dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut packet = self
            .book_packet
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if packet.is_none() {
            *packet = build(&recipes);
        }
        packet.clone()
    }

    pub fn add_recipe(&self, recipe: DynamicRecipe) {
        let mut recipes = self.change();
        recipes.push(recipe);
    }

    pub fn add_recipes(&self, new_recipes: impl IntoIterator<Item = DynamicRecipe>) {
        let mut recipes = self.change();
        recipes.extend(new_recipes);
    }

    pub fn set_recipes(&self, new_recipes: Vec<DynamicRecipe>) {
        let mut recipes = self.change();
        *recipes = new_recipes;
    }

    pub fn clear(&self) {
        let mut recipes = self.change();
        recipes.clear();
    }

    pub fn get_dynamic_recipes_internal(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl RecipeProvider for RecipeManager {
    fn get_dynamic_recipes(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_recipe_book_packet_is_built_once_until_the_recipes_change() {
        let manager = RecipeManager::new();
        let built = std::cell::Cell::new(0u8);
        let packet = || {
            manager.book_packet(|_| {
                built.set(built.get() + 1);
                Some(Bytes::from(vec![built.get()]))
            })
        };
        assert_eq!(packet().as_deref(), Some(&[1][..]));
        assert_eq!(packet().as_deref(), Some(&[1][..]));

        manager.set_recipes(Vec::new());
        assert_eq!(packet().as_deref(), Some(&[2][..]));
        manager.clear();
        assert_eq!(packet().as_deref(), Some(&[3][..]));
        manager.add_recipes(Vec::new());
        assert_eq!(packet().as_deref(), Some(&[4][..]));
        assert_eq!(packet().as_deref(), Some(&[4][..]));
    }

    #[test]
    fn a_packet_that_could_not_be_built_is_tried_again() {
        let manager = RecipeManager::new();
        assert_eq!(manager.book_packet(|_| None), None);
        let packet = manager.book_packet(|_| Some(Bytes::from_static(b"ok")));
        assert_eq!(packet.as_deref(), Some(&b"ok"[..]));
    }
}
