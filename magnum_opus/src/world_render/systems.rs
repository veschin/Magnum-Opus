use super::palette::{
    GRID_HALF, TILE_WORLD_SIZE, VEIN_RADIUS, cell_top_height, resource_linear,
    terrain_base_y, terrain_linear, terrain_top_y, tile_center_xz,
};
use super::resource::WorldSceneCache;
use crate::landscape::{Landscape, TerrainCell, TerrainKind};
use crate::render_pipeline::{SCENE_LAYER, ToonMaterial, ToonParams};
use crate::resources::{ResourceKind, ResourceVeins, Vein};
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

    let _ = GRID_HALF;

    let width = landscape.width;
    let height = landscape.height;
    let cells: Vec<TerrainCell> = landscape.cells.clone();
    let vein_map: BTreeMap<(u32, u32), Vein> = veins.veins.clone();

    // Deferred closure - View systems must stay ReadOnlySystemParam, so asset
    // mutation and entity spawning happen here where `&mut World` is granted.
    commands.queue(move |world: &mut World| {
        let scene_layer = RenderLayers::layer(SCENE_LAYER);

        // Single unit cuboid (1×1×1) is reused for every tile; per-cell height
        // comes from `Transform::scale.y`. One material per TerrainKind carries
        // the flat toon colour.
        let tile_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::new(TILE_WORLD_SIZE, 1.0, TILE_WORLD_SIZE));
        let mut terrain_materials: BTreeMap<TerrainKind, Handle<ToonMaterial>> =
            BTreeMap::new();

        let mut new_tiles: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
        for y in 0..height {
            for x in 0..width {
                let cell = cells[(y * width + x) as usize];
                let material = match terrain_materials.get(&cell.kind) {
                    Some(h) => h.clone(),
                    None => {
                        let mat = ToonMaterial {
                            params: ToonParams {
                                base_color: terrain_linear(cell.kind),
                                ..ToonParams::default()
                            },
                        };
                        let h = world.resource_mut::<Assets<ToonMaterial>>().add(mat);
                        terrain_materials.insert(cell.kind, h.clone());
                        h
                    }
                };
                let h = cell_top_height(cell);
                let base = terrain_base_y(cell.kind);
                let (cx, cz) = tile_center_xz(x, y);
                let translation = Vec3::new(cx, base + h / 2.0, cz);
                let entity = world
                    .spawn((
                        Mesh3d(tile_mesh.clone()),
                        MeshMaterial3d(material),
                        Transform::from_translation(translation)
                            .with_scale(Vec3::new(1.0, h, 1.0)),
                        scene_layer.clone(),
                    ))
                    .id();
                new_tiles.insert((x, y), entity);
            }
        }

        let vein_mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Sphere::new(VEIN_RADIUS));
        let mut resource_materials: BTreeMap<ResourceKind, Handle<ToonMaterial>> =
            BTreeMap::new();

        let mut new_veins: BTreeMap<(u32, u32), Entity> = BTreeMap::new();
        for ((x, y), vein) in &vein_map {
            let cell = cells[(*y * width + *x) as usize];
            let material = match resource_materials.get(&vein.kind) {
                Some(h) => h.clone(),
                None => {
                    let mat = ToonMaterial {
                        params: ToonParams {
                            base_color: resource_linear(vein.kind),
                            ..ToonParams::default()
                        },
                    };
                    let h = world.resource_mut::<Assets<ToonMaterial>>().add(mat);
                    resource_materials.insert(vein.kind, h.clone());
                    h
                }
            };
            let (cx, cz) = tile_center_xz(*x, *y);
            let pos = Vec3::new(cx, terrain_top_y(cell) + VEIN_RADIUS, cz);
            let entity = world
                .spawn((
                    Mesh3d(vein_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation(pos),
                    scene_layer.clone(),
                ))
                .id();
            new_veins.insert((*x, *y), entity);
        }

        world.insert_resource(WorldSceneCache {
            tiles: new_tiles,
            veins: new_veins,
            synced: true,
        });
    });
}
