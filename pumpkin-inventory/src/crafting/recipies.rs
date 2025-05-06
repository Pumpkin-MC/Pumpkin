use pumpkin_world::inventory::inventory::Inventory;

// RecipeMatcher.java
pub struct RecipeMatcher {}

// RecipeFinder.java
pub struct RecipeFinder {}

// AbstractRecipeScreenHandle.java
pub trait RecipeFinderScreenHandler {}

pub trait RecipeInputInventory: Inventory {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;
    //fn get_held_stacks(), Get a lock on the inventory instead
    // createRecipeInput
    // createPositionedRecipeInput
}
