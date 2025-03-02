use serde::Deserialize;

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum NormalInvProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformIntProvider),
    // TODO: Add more...
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
pub enum InvProvider {
    Object(NormalInvProvider),
    Constant(i32),
}

impl InvProvider {
    pub fn get_min(&self) -> i32 {
        match self {
            InvProvider::Object(inv_provider) => match inv_provider {
                NormalInvProvider::Uniform(uniform) => uniform.get_min(),
            },
            InvProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self) -> i32 {
        match self {
            InvProvider::Object(inv_provider) => match inv_provider {
                NormalInvProvider::Uniform(uniform) => uniform.get(),
            },
            InvProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> i32 {
        match self {
            InvProvider::Object(inv_provider) => match inv_provider {
                NormalInvProvider::Uniform(uniform) => uniform.get_max(),
            },
            InvProvider::Constant(i) => *i,
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
