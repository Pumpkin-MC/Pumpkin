#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_set_jigsaw_block(&self, player: &Arc<Player>, jigsaw: &SSetJigsawBlock<'_>) {
        if !player.is_creative() {
            return;
        }
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }
        let pos = jigsaw.pos;
        let block_entity = player.world().get_block_entity(&pos);
        if let Some(block_entity) = block_entity {
            if block_entity.resource_location() != JigsawBlockEntity::ID {
                warn!("Client tried to change Jigsaw block but not Jigsaw block entity found");
                return;
            }

            let Some(jigsaw_block) = block_entity.as_any().downcast_ref::<JigsawBlockEntity>()
            else {
                return;
            };

            *jigsaw_block
                .name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = jigsaw.name.to_string();
            *jigsaw_block
                .target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = jigsaw.target.to_string();
            *jigsaw_block
                .pool
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = jigsaw.pool.to_string();
            *jigsaw_block
                .final_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                jigsaw.final_state.to_string();
            *jigsaw_block
                .joint
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                JigsawJointType::from_str(jigsaw.joint);
            jigsaw_block
                .selection_priority
                .store(jigsaw.selection_priority.0, Ordering::SeqCst);
            jigsaw_block
                .placement_priority
                .store(jigsaw.placement_priority.0, Ordering::SeqCst);
            jigsaw_block.dirty.store(true, Ordering::Relaxed);

            player.world().update_block_entity(&block_entity);
        }
    }
}
