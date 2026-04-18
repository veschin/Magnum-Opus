//! Diff-sync cuboid meshes for Building entities on the scene layer.
//! Buildings stand on top of their underlying tile (height-aware) and use the
//! shared `ToonMaterial` so they receive the same flat banded shading as the
//! terrain underneath.

use super::palette::{BUILDING_WORLD_SIZE, building_height, building_linear};
use super::resource::BuildingSceneCache;
use crate::buildings::{Building, BuildingType};
use crate::grid::Position;
use crate::landscape::Landscape;
use crate::render_pipeline::{SCENE_LAYER, ToonMaterial, ToonParams};
use crate::world_render::palette::{terrain_top_y, tile_center_xz};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn building_render_system(
    mut commands: Commands,
    cache: Res<BuildingSceneCache>,
    landscape: Res<Landscape>,
    buildings_q: Query<(Entity, &Building, &Position)>,
) {
    let buildings_snapshot: Vec<(Entity, BuildingType, Position)> = buildings_q
        .iter()
        .map(|(e, b, p)| (e, b.building_type, *p))
        .collect();
    let cache_snapshot: BTreeMap<Entity, Entity> = cache.entities.clone();
    let width = landscape.width;
    let cells = landscape.cells.clone();

    commands.queue(move |world: &mut World| {
        let scene_layer = RenderLayers::layer(SCENE_LAYER);
        let mut new_entities = cache_snapshot.clone();
        let mut changed = false;

        let current: BTreeSet<Entity> =
            buildings_snapshot.iter().map(|(e, _, _)| *e).collect();

        let mut stale = Vec::new();
        for (&building, &sprite) in &cache_snapshot {
            if !current.contains(&building) {
                if let Ok(entity_mut) = world.get_entity_mut(sprite) {
                    entity_mut.despawn();
                }
                stale.push(building);
                changed = true;
            }
        }
        for b in stale {
            new_entities.remove(&b);
        }

        let mut type_assets: BTreeMap<
            BuildingType,
            (Handle<Mesh>, Handle<ToonMaterial>),
        > = BTreeMap::new();

        for (entity, btype, position) in &buildings_snapshot {
            if new_entities.contains_key(entity) {
                continue;
            }
            let assets = match type_assets.get(btype) {
                Some(pair) => pair.clone(),
                None => {
                    let height = building_height(*btype);
                    let mesh = world.resource_mut::<Assets<Mesh>>().add(Cuboid::new(
                        BUILDING_WORLD_SIZE,
                        height,
                        BUILDING_WORLD_SIZE,
                    ));
                    let mat = ToonMaterial {
                        params: ToonParams {
                            base_color: building_linear(*btype),
                            ..ToonParams::default()
                        },
                    };
                    let material =
                        world.resource_mut::<Assets<ToonMaterial>>().add(mat);
                    let pair = (mesh, material);
                    type_assets.insert(*btype, pair.clone());
                    pair
                }
            };
            // Building sits on top of its tile; the cell is read from the
            // landscape so the per-cell elevation jitter is respected.
            let cell = if !cells.is_empty() && position.x < width {
                cells[(position.y * width + position.x) as usize]
            } else {
                crate::landscape::TerrainCell::default()
            };
            let ground_top = terrain_top_y(cell);
            let (cx, cz) = tile_center_xz(position.x, position.y);
            let world_pos =
                Vec3::new(cx, ground_top + building_height(*btype) / 2.0, cz);
            let sprite = world
                .spawn((
                    Mesh3d(assets.0),
                    MeshMaterial3d(assets.1),
                    Transform::from_translation(world_pos),
                    scene_layer.clone(),
                ))
                .id();
            new_entities.insert(*entity, sprite);
            changed = true;
        }

        if changed {
            world.insert_resource(BuildingSceneCache {
                entities: new_entities,
            });
        }
    });
}
