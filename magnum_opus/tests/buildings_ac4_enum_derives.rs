//! F4 AC4 - BuildingType derives Hash + Ord so it works as key in both map kinds.

use magnum_opus::buildings::BuildingType;
use std::collections::{BTreeMap, HashMap};

#[test]
fn building_type_works_as_btreemap_and_hashmap_key() {
    let mut b: BTreeMap<BuildingType, u32> = BTreeMap::new();
    b.insert(BuildingType::Miner, 1);
    b.insert(BuildingType::Smelter, 2);
    b.insert(BuildingType::Mall, 3);
    b.insert(BuildingType::EnergySource, 4);
    assert_eq!(b.len(), 4);

    let mut h: HashMap<BuildingType, u32> = HashMap::new();
    h.insert(BuildingType::Miner, 1);
    h.insert(BuildingType::Smelter, 2);
    h.insert(BuildingType::Mall, 3);
    h.insert(BuildingType::EnergySource, 4);
    assert_eq!(h.len(), 4);
}
