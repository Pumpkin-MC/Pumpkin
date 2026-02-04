pub type ResourceLocation = String;

pub trait ToResourceLocation: Sized {
    fn to_resource_location(&self) -> ResourceLocation;
}

pub trait FromResourceLocation: Sized {
    fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self>;
}
