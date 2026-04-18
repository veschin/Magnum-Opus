//! F19 world-render / AC3: cache collections are BTreeMap<(u32,u32), Entity>.

use bevy::prelude::Entity;
use magnum_opus::world_render::WorldSceneCache;
use std::collections::BTreeMap;

#[test]
fn ac3_cache_uses_btreemap_for_both_collections() {
    let cache = WorldSceneCache::default();
    let _tiles: &BTreeMap<(u32, u32), Entity> = &cache.tiles;
    let _veins: &BTreeMap<(u32, u32), Entity> = &cache.veins;
    assert!(cache.tiles.is_empty());
    assert!(cache.veins.is_empty());
}
