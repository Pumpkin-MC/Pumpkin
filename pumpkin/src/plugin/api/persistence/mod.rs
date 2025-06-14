use uuid::Uuid;

mod persistent_data_container;

pub trait HasUuid {
    fn get_uuid(&self) -> Uuid;
}
