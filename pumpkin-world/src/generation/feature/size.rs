pub struct FeatureSize {
    pub min_clipped_height: Option<u8>,
    pub r#type: FeatureSizeType,
}

pub enum FeatureSizeType {
    ThreeLayersFeatureSize(ThreeLayersFeatureSize),
    TwoLayersFeatureSize(TwoLayersFeatureSize),
}

impl FeatureSizeType {
    pub const fn get_radius(&self, height: u32, y: i32) -> i32 {
        match self {
            Self::ThreeLayersFeatureSize(three) => three.get_radius(height, y),
            Self::TwoLayersFeatureSize(two) => two.get_radius(y),
        }
    }
}

pub struct TwoLayersFeatureSize {
    pub limit: u8,
    pub lower_size: u8,
    pub upper_size: u8,
}

impl TwoLayersFeatureSize {
    pub const fn get_radius(&self, y: i32) -> i32 {
        if y < self.limit as i32 {
            self.lower_size as i32
        } else {
            self.upper_size as i32
        }
    }
}

pub struct ThreeLayersFeatureSize {
    pub limit: u8,
    pub upper_limit: u8,
    pub lower_size: u8,
    pub middle_size: u8,
    pub upper_size: u8,
}

impl ThreeLayersFeatureSize {
    pub const fn get_radius(&self, height: u32, y: i32) -> i32 {
        if y < self.limit as i32 {
            self.lower_size as i32
        } else if y >= height as i32 - self.upper_limit as i32 {
            self.upper_size as i32
        } else {
            self.middle_size as i32
        }
    }
}
