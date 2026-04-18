use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Resource;

use super::contract::SimContract;
use super::phase::Phase;
use super::type_key::TypeKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Archetype {
    Sim,
    Data,
    View,
    Input,
    Core,
}

#[derive(Debug, Clone)]
pub struct ModuleRecord {
    pub id: &'static str,
    pub archetype: Archetype,
    pub phase: Option<Phase>,
    pub reads: Vec<TypeKey>,
    pub writes: Vec<TypeKey>,
    pub commands_in: Vec<TypeKey>,
    pub commands_out: Vec<TypeKey>,
    pub messages_in: Vec<TypeKey>,
    pub messages_out: Vec<TypeKey>,
}

/// Tracks every registered module and enforces cross-cutting invariants.
///
/// Keyed on `TypeId` for identity (so `a::Grid` and `b::Grid` never collide).
/// Stores `TypeKey` alongside owners for error diagnostics.
///
/// Frozen by `finalize_checks` - any further `register_*` after finalize panics.
#[derive(Resource, Default, Debug)]
pub struct ModuleRegistry {
    modules: BTreeMap<&'static str, ModuleRecord>,
    writers: BTreeMap<TypeId, (TypeKey, &'static str)>,
    finalized: bool,
}

impl ModuleRegistry {
    pub fn register_sim(&mut self, id: &'static str, phase: Phase, c: &SimContract) {
        self.check_not_finalized(id);
        self.check_unique_id(id);
        self.check_phase_not_reserved(id, &phase);
        for w in c.writes {
            self.claim_writer(id, *w);
        }
        self.modules.insert(
            id,
            ModuleRecord {
                id,
                archetype: Archetype::Sim,
                phase: Some(phase),
                reads: c.reads.to_vec(),
                writes: c.writes.to_vec(),
                commands_in: c.commands_in.to_vec(),
                commands_out: Vec::new(),
                messages_in: c.messages_in.to_vec(),
                messages_out: c.messages_out.to_vec(),
            },
        );
    }

    pub fn register_data(&mut self, id: &'static str, writes: &[TypeKey]) {
        self.check_not_finalized(id);
        self.check_unique_id(id);
        for w in writes {
            self.claim_writer(id, *w);
        }
        self.modules.insert(
            id,
            ModuleRecord {
                id,
                archetype: Archetype::Data,
                phase: None,
                reads: Vec::new(),
                writes: writes.to_vec(),
                commands_in: Vec::new(),
                commands_out: Vec::new(),
                messages_in: Vec::new(),
                messages_out: Vec::new(),
            },
        );
    }

    pub fn register_view(&mut self, id: &'static str, reads: &[TypeKey], writes: &[TypeKey]) {
        self.check_not_finalized(id);
        self.check_unique_id(id);
        for w in writes {
            self.claim_writer(id, *w);
        }
        self.modules.insert(
            id,
            ModuleRecord {
                id,
                archetype: Archetype::View,
                phase: None,
                reads: reads.to_vec(),
                writes: writes.to_vec(),
                commands_in: Vec::new(),
                commands_out: Vec::new(),
                messages_in: Vec::new(),
                messages_out: Vec::new(),
            },
        );
    }

    pub fn register_input(
        &mut self,
        id: &'static str,
        reads: &[TypeKey],
        writes: &[TypeKey],
        commands_out: &[TypeKey],
    ) {
        self.check_not_finalized(id);
        self.check_unique_id(id);
        for w in writes {
            self.claim_writer(id, *w);
        }
        self.modules.insert(
            id,
            ModuleRecord {
                id,
                archetype: Archetype::Input,
                phase: None,
                reads: reads.to_vec(),
                writes: writes.to_vec(),
                commands_in: Vec::new(),
                commands_out: commands_out.to_vec(),
                messages_in: Vec::new(),
                messages_out: Vec::new(),
            },
        );
    }

    /// Claim a resource for the core runtime (Tick, ModuleRegistry, MetricsRegistry).
    /// Enters the writer table under owner `"core"` so any module trying to claim
    /// the same type triggers a single-writer violation.
    ///
    /// Crate-private: only `CorePlugin::build` may call this.
    pub(crate) fn register_core_writer(&mut self, resource: TypeKey) {
        self.check_not_finalized("core");
        self.claim_writer("core", resource);
        let entry = self.modules.entry("core").or_insert_with(|| ModuleRecord {
            id: "core",
            archetype: Archetype::Core,
            phase: None,
            reads: Vec::new(),
            writes: Vec::new(),
            commands_in: Vec::new(),
            commands_out: Vec::new(),
            messages_in: Vec::new(),
            messages_out: Vec::new(),
        });
        entry.writes.push(resource);
    }

    fn check_not_finalized(&self, id: &'static str) {
        if self.finalized {
            panic!(
                "module-registry: module {:?} registered after finalize_modules() - registry is frozen",
                id,
            );
        }
    }

    fn check_unique_id(&self, id: &'static str) {
        if self.modules.contains_key(id) {
            panic!("module-registry: duplicate module id {:?}", id);
        }
    }

    /// Reject `PRIMARY_PHASE` values in the core-reserved set.
    /// `Phase::Commands` is reserved for `add_command_drain`, `Phase::Metrics`
    /// for `add_metric_publish`, `Phase::End` for the core tick increment.
    /// Sim modules must own one of the logical phases (World..Progression).
    fn check_phase_not_reserved(&self, id: &'static str, phase: &Phase) {
        match phase {
            Phase::Commands | Phase::Metrics | Phase::End => panic!(
                "module-registry: module {:?} declares PRIMARY_PHASE={:?} which is reserved by the core runtime",
                id, phase,
            ),
            _ => {}
        }
    }

    fn claim_writer(&mut self, id: &'static str, resource: TypeKey) {
        if let Some((existing_key, existing_owner)) = self.writers.get(&resource.id) {
            panic!(
                "module-registry: single-writer violation on {:?} (also seen as {:?}) - owned by {:?}, also claimed by {:?}",
                resource.name, existing_key.name, existing_owner, id,
            );
        }
        self.writers.insert(resource.id, (resource, id));
    }

    /// Verify closures of commands, messages, and reads across all registered modules.
    ///
    /// Checks:
    /// - closed-messages: every `messages_in` has at least one `messages_out`.
    /// - single-producer-messages: every `messages_out` appears in exactly one module.
    /// - closed-commands: every `commands_in` has a matching `commands_out`.
    /// - single-producer-commands: every `commands_out` appears in exactly one module.
    /// - single-consumer-commands: every `commands_in` appears in exactly one module.
    /// - closed-reads: every `reads` has a matching `writes` somewhere.
    ///
    /// On success, marks the registry as frozen - subsequent `register_*` panics.
    pub fn finalize_checks(&mut self) -> Result<(), Vec<String>> {
        let mut msg_producers: BTreeMap<TypeId, Vec<(TypeKey, &'static str)>> = BTreeMap::new();
        let mut cmd_producers: BTreeMap<TypeId, Vec<(TypeKey, &'static str)>> = BTreeMap::new();
        let mut cmd_consumers: BTreeMap<TypeId, Vec<(TypeKey, &'static str)>> = BTreeMap::new();
        let mut written: BTreeSet<TypeId> = BTreeSet::new();

        for m in self.modules.values() {
            for msg in &m.messages_out {
                msg_producers.entry(msg.id).or_default().push((*msg, m.id));
            }
            for cmd in &m.commands_out {
                cmd_producers.entry(cmd.id).or_default().push((*cmd, m.id));
            }
            for cmd in &m.commands_in {
                cmd_consumers.entry(cmd.id).or_default().push((*cmd, m.id));
            }
            for w in &m.writes {
                written.insert(w.id);
            }
        }

        let mut errs = Vec::new();

        for m in self.modules.values() {
            for want in &m.messages_in {
                if !msg_producers.contains_key(&want.id) {
                    errs.push(format!(
                        "closed-messages: module {:?} reads message {:?} with no registered producer",
                        m.id, want.name,
                    ));
                }
            }
            for want in &m.commands_in {
                if !cmd_producers.contains_key(&want.id) {
                    errs.push(format!(
                        "closed-commands: module {:?} drains command {:?} with no registered producer",
                        m.id, want.name,
                    ));
                }
            }
            for want in &m.reads {
                if !written.contains(&want.id) {
                    errs.push(format!(
                        "closed-reads: module {:?} reads resource {:?} with no registered writer",
                        m.id, want.name,
                    ));
                }
            }
        }

        for producers in msg_producers.values() {
            if producers.len() > 1 {
                let owners: Vec<&'static str> = producers.iter().map(|(_, id)| *id).collect();
                let name = producers[0].0.name;
                errs.push(format!(
                    "single-producer-messages: message {:?} has multiple producers: {:?}",
                    name, owners,
                ));
            }
        }
        for producers in cmd_producers.values() {
            if producers.len() > 1 {
                let owners: Vec<&'static str> = producers.iter().map(|(_, id)| *id).collect();
                let name = producers[0].0.name;
                errs.push(format!(
                    "single-producer-commands: command {:?} has multiple producers: {:?}",
                    name, owners,
                ));
            }
        }
        for consumers in cmd_consumers.values() {
            if consumers.len() > 1 {
                let owners: Vec<&'static str> = consumers.iter().map(|(_, id)| *id).collect();
                let name = consumers[0].0.name;
                errs.push(format!(
                    "single-consumer-commands: command {:?} has multiple consumers: {:?}",
                    name, owners,
                ));
            }
        }

        if errs.is_empty() {
            self.finalized = true;
            Ok(())
        } else {
            Err(errs)
        }
    }

    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    pub fn get(&self, id: &'static str) -> Option<&ModuleRecord> {
        self.modules.get(id)
    }

    pub fn modules(&self) -> impl Iterator<Item = &ModuleRecord> {
        self.modules.values()
    }

    pub fn writer_of(&self, resource: TypeId) -> Option<&'static str> {
        self.writers.get(&resource).map(|(_, owner)| *owner)
    }

    pub fn writer_of_type<T: 'static>(&self) -> Option<&'static str> {
        self.writer_of(TypeId::of::<T>())
    }

    pub fn len(&self) -> usize {
        self.modules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}
