use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NormalIntProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformIntProvider),
    // TODO: Add more...
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum IntProvider {
    Object(NormalIntProvider),
    Constant(i32),
}

impl IntProvider {
    pub fn get_min(&self) -> i32 {
        match self {
            IntProvider::Object(inv_provider) => match inv_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get_min(),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self) -> i32 {
        match self {
            IntProvider::Object(inv_provider) => match inv_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get(),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> i32 {
        match self {
            IntProvider::Object(inv_provider) => match inv_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get_max(),
            },
            IntProvider::Constant(i) => *i,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct UniformIntProvider {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl UniformIntProvider {
    pub fn get_min(&self) -> i32 {
        self.min_inclusive
    }
    pub fn get(&self) -> i32 {
        rand::random_range(self.min_inclusive..self.max_inclusive)
    }
    pub fn get_max(&self) -> i32 {
        self.max_inclusive
    }
}
