mod configured_features;
/// So first we go trough all the placed features and check if we should place a feature somewhere using `placed_features`.
/// then if we want to place a feature we place it using the `configured_features`, there is the logic for how we are going to place the feature
pub mod placed_features;

mod features;
mod size;
