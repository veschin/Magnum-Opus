use std::collections::BTreeMap;

use bevy::prelude::Resource;

use super::contract::{MetricDesc, MetricKind};

#[derive(Debug, Clone)]
pub struct MetricEntry {
    pub owner: &'static str,
    pub desc: MetricDesc,
    pub value: f64,
}

/// Global metrics registry. One entry per metric name.
/// Names must be unique across all modules.
#[derive(Resource, Default)]
pub struct MetricsRegistry {
    entries: BTreeMap<&'static str, MetricEntry>,
}

impl MetricsRegistry {
    pub fn declare(&mut self, owner: &'static str, desc: MetricDesc) {
        if let Some(existing) = self.entries.get(desc.name) {
            panic!(
                "metrics: duplicate metric name {:?} (owner={:?}, existing owner={:?})",
                desc.name, owner, existing.owner,
            );
        }
        self.entries.insert(
            desc.name,
            MetricEntry {
                owner,
                desc,
                value: 0.0,
            },
        );
    }

    pub fn set(&mut self, name: &'static str, value: f64) {
        let e = self
            .entries
            .get_mut(name)
            .unwrap_or_else(|| panic!("metrics: unknown metric name {:?}", name));
        e.value = value;
    }

    pub fn inc(&mut self, name: &'static str, delta: f64) {
        let e = self
            .entries
            .get_mut(name)
            .unwrap_or_else(|| panic!("metrics: unknown metric name {:?}", name));
        assert!(
            matches!(e.desc.kind, MetricKind::Counter),
            "metrics: inc() on non-counter metric {:?} (kind={:?})",
            name,
            e.desc.kind,
        );
        e.value += delta;
    }

    pub fn get(&self, name: &'static str) -> Option<f64> {
        self.entries.get(name).map(|e| e.value)
    }

    pub fn owner(&self, name: &'static str) -> Option<&'static str> {
        self.entries.get(name).map(|e| e.owner)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&&'static str, &MetricEntry)> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
