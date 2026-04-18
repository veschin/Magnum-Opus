//! Diff-sync sprites for Building entities on the scene layer.

use super::palette::{BUILDING_PX, building_color, tile_world_pos};
use super::resource::BuildingSceneCache;
use crate::buildings::Building;
use crate::grid::Position;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn building_render_system(
    mut commands: Commands,
    cache: Res<BuildingSceneCache>,
    buildings_q: Query<(Entity, &Building, &Position)>,
) {
    let scene_layer = RenderLayers::layer(1);
    let mut new_entities: BTreeMap<Entity, Entity> = cache.entities.clone();
    let mut changed = false;

    let current: BTreeSet<Entity> = buildings_q.iter().map(|(e, _, _)| e).collect();

    let mut stale = Vec::new();
    for (&building, &sprite) in &cache.entities {
        if !current.contains(&building) {
            commands.entity(sprite).despawn();
            stale.push(building);
            changed = true;
        }
    }
    for b in stale {
        new_entities.remove(&b);
    }

    for (entity, building, position) in buildings_q.iter() {
        if new_entities.contains_key(&entity) {
            continue;
        }
        let world_pos = tile_world_pos(position.x, position.y) + Vec3::new(0.0, 0.0, 0.2);
        let sprite = commands
            .spawn((
                Sprite::from_color(
                    building_color(building.building_type),
                    Vec2::splat(BUILDING_PX),
                ),
                Transform::from_translation(world_pos),
                scene_layer.clone(),
            ))
            .id();
        new_entities.insert(entity, sprite);
        changed = true;
    }

    if changed {
        commands.insert_resource(BuildingSceneCache {
            entities: new_entities,
        });
    }
}
