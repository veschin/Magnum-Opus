//! F19 world-render / AC2: default cache state is unsynced and empty.

use magnum_opus::world_render::WorldSceneCache;

#[test]
fn ac2_default_cache_is_unsynced_and_empty() {
    let cache = WorldSceneCache::default();
    assert!(!cache.synced);
    assert!(cache.tiles.is_empty());
    assert!(cache.veins.is_empty());
}
