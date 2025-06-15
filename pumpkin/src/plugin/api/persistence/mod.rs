use uuid::Uuid;

mod persistent_data_container;

pub trait HasUuid {
    fn get_uuid(&self) -> Uuid;

    fn get_by_uuid(uuid: &Uuid) -> &'static Self
    where
        Self: Sized,
    {
        todo!("bro honestly idk")
    }
}
