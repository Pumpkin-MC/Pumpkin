use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NormalFloatProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformFloatProvider),
    // TODO: Add more...
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum FloatProvider {
    Object(NormalFloatProvider),
    Constant(f32),
}

impl FloatProvider {
    pub fn get_min(&self) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Uniform(uniform) => uniform.get_min(),
            },
            FloatProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Uniform(uniform) => uniform.get(),
            },
            FloatProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> f32 {
        match self {
            FloatProvider::Object(inv_provider) => match inv_provider {
                NormalFloatProvider::Uniform(uniform) => uniform.get_max(),
            },
            FloatProvider::Constant(i) => *i,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct UniformFloatProvider {
    min_inclusive: f32,
    max_inclusive: f32,
}

impl UniformFloatProvider {
    pub fn get_min(&self) -> f32 {
        self.min_inclusive
    }
    pub fn get(&self) -> f32 {
        rand::random_range(self.min_inclusive..self.max_inclusive)
    }
    pub fn get_max(&self) -> f32 {
        self.max_inclusive
    }
}
