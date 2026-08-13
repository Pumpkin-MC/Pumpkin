#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_jigsaw_generate(&self, player: &Arc<Player>, generate: SJigsawGenerate) {
        if !player.is_creative() {
            return;
        }
        if player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }
        let pos = generate.pos;
        if let Some(block_entity) = player.world().get_block_entity(&pos)
            && let Some(jigsaw_block) = block_entity.as_any().downcast_ref::<JigsawBlockEntity>()
        {
            jigsaw_block
                .generate(&player.world(), generate.levels.0, generate.keep_jigsaws)
                .await;
        }
    }
}
