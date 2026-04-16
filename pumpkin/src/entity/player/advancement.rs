use pumpkin_data::Advancement;
use crate::entity::NBTStorage;

pub struct AdvancementProgress {
    pub complete: bool,
}

impl AdvancementProgress {

    pub fn is_done(&self) -> bool {
        self.complete
    }

    pub fn has_progress(&self) -> bool {
        self.complete
    }
}

pub struct PlayerAdvancement {
    
}

impl PlayerAdvancement {
    pub(crate) fn new() -> Self {
        PlayerAdvancement {}
    }

    pub fn flush_dirty(&self, flush: bool) {
        todo!()
    }

    pub fn get_or_start_progress(&self,advancement:&Advancement) -> AdvancementProgress{
        todo!()
    }

    pub fn award(&self,advancement:&Advancement){}

    pub fn revoke(&self,advancement:&Advancement){}
}

impl NBTStorage for PlayerAdvancement {

}