use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeatureSize {
    pub min_clipped_height: Option<u8>,
    #[serde(flatten)]
    pub r#type: FeatureSizeType,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum FeatureSizeType {
    #[serde(rename = "minecraft:three_layers_feature_size")]
    ThreeLayersFeatureSize(ThreeLayersFeatureSize),
    #[serde(rename = "minecraft:two_layers_feature_size")]
    TwoLayersFeatureSize(TwoLayersFeatureSize),
}

impl FeatureSizeType {
    pub fn get_radius(&self, height: u32, y: i32) -> i32 {
        match self {
            FeatureSizeType::ThreeLayersFeatureSize(three) => three.get_radius(height, y),
            FeatureSizeType::TwoLayersFeatureSize(two) => two.get_radius(y),
        }
    }
}

#[derive(Deserialize)]
pub struct TwoLayersFeatureSize {
    limit: u8,
    lower_size: u8,
    upper_size: u8,
}

impl TwoLayersFeatureSize {
    pub fn get_radius(&self, y: i32) -> i32 {
        if y < self.limit as i32 {
            self.lower_size as i32
        } else {
            self.upper_size as i32
        }
    }
}

#[derive(Deserialize)]
pub struct ThreeLayersFeatureSize {
    limit: u8,
    upper_limit: u8,
    lower_size: u8,
    middle_size: u8,
    upper_size: u8,
}

impl ThreeLayersFeatureSize {
    pub fn get_radius(&self, height: u32, y: i32) -> i32 {
        if y < self.limit as i32 {
            self.lower_size as i32
        } else if y >= height as i32 - self.upper_limit as i32 {
            self.upper_size as i32
        } else {
            self.middle_size as i32
        }
    }
}
