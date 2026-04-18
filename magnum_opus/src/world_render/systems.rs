use super::palette::{TILE_PX, VEIN_PX, resource_color, terrain_color, tile_world_pos};
use super::resource::WorldSceneCache;
use crate::landscape::Landscape;
use crate::resources::ResourceVeins;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use std::collections::BTreeMap;

pub fn world_render_system(
    mut commands: Commands,
    cache: Res<WorldSceneCache>,
    landscape: Res<Landscape>,
    veins: Res<ResourceVeins>,
) {
    if cache.synced {
        return;
    }
    if !landscape.ready || !veins.ready {
        return;
    }

    let scene_layer = RenderLayers::layer(1);
    let mut new_tiles: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
    for y in 0..landscape.height {
        for x in 0..landscape.width {
            let cell = landscape.cells[(y * landscape.width + x) as usize];
            let pos = tile_world_pos(x, y);
            let entity = commands
                .spawn((
                    Sprite::from_color(terrain_color(cell.kind), Vec2::splat(TILE_PX)),
                    Transform::from_translation(pos),
                    scene_layer.clone(),
                ))
                .id();
            new_tiles.insert((x, y), entity);
        }
    }

    let mut new_veins: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
    for (&(x, y), vein) in &veins.veins {
        let pos = tile_world_pos(x, y) + Vec3::new(0.0, 0.0, 0.1);
        let entity = commands
            .spawn((
                Sprite::from_color(resource_color(vein.kind), Vec2::splat(VEIN_PX)),
                Transform::from_translation(pos),
                scene_layer.clone(),
            ))
            .id();
        new_veins.insert((x, y), entity);
    }

    commands.insert_resource(WorldSceneCache {
        tiles: new_tiles,
        veins: new_veins,
        synced: true,
    });
}
