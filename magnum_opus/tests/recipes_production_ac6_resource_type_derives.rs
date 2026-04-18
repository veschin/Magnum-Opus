//! F5a AC6 - ResourceType derives Hash + Ord.

use magnum_opus::recipes_production::ResourceType;
use std::collections::{BTreeMap, HashMap};

#[test]
fn resource_type_works_in_both_maps() {
    let mut b: BTreeMap<ResourceType, u32> = BTreeMap::new();
    b.insert(ResourceType::Wood, 1);
    b.insert(ResourceType::Stone, 2);
    b.insert(ResourceType::IronOre, 3);
    b.insert(ResourceType::IronBar, 4);
    b.insert(ResourceType::Coal, 5);
    assert_eq!(b.len(), 5);

    let mut h: HashMap<ResourceType, u32> = HashMap::new();
    h.insert(ResourceType::Wood, 1);
    h.insert(ResourceType::Stone, 2);
    h.insert(ResourceType::IronOre, 3);
    h.insert(ResourceType::IronBar, 4);
    h.insert(ResourceType::Coal, 5);
    assert_eq!(h.len(), 5);
}
