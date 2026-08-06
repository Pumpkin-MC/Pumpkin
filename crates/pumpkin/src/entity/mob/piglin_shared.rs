use crate::entity::{EntityBase, mob::Mob};
use pumpkin_data::entity::EntityType;

const ALERT_RADIUS: f64 = 16.0;

/// Sets `mob` to retaliate against `source`, then alerts nearby adult piglins that
/// don't already have a target to retaliate too.
///
/// Goal-based approximation of `PiglinAi.maybeRetaliate` + `broadcastAngerTarget`
/// (PiglinAi.java:556-618): vanilla propagates via a `Brain` memory broadcast to
/// `NEARBY_ADULT_PIGLINS`, which Pumpkin has no equivalent of, so this does a direct
/// 16-block world query instead (same pattern `RevengeGoal::start` already uses for
/// same-type alerting). Shared by `PiglinEntity` and `PiglinBruteEntity::on_damage`,
/// matching vanilla's shared `PiglinAi.maybeRetaliate` call from both `PiglinAi.wasHurtBy`
/// and `PiglinBruteAi.wasHurtBy`.
pub async fn retaliate_and_alert_piglins(mob: &dyn Mob, source: &dyn EntityBase) {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let world = entity.world.load();

    let source_id = source.get_entity().entity_id;
    let Some(source_arc) = world.get_entity_by_id(source_id) else {
        return;
    };

    mob.set_mob_target(Some(source_arc.clone())).await;

    let position = entity.pos.load();
    for nearby in world
        .get_nearby_entities(position, ALERT_RADIUS)
        .into_values()
    {
        if nearby.get_entity().entity_id == entity.entity_id
            || nearby.get_entity().entity_type.id != EntityType::PIGLIN.id
        {
            continue;
        }
        let Some(nearby_mob) = nearby.get_mob() else {
            continue;
        };
        if nearby_mob.get_mob_entity().target.lock().await.is_some() {
            continue;
        }
        nearby_mob.set_mob_target(Some(source_arc.clone())).await;
    }
}
