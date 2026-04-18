//! Install-time contexts for the four module archetypes.
//!
//! The core never hands `&mut App` to module code. Instead, each archetype
//! receives a scoped installer that exposes only the operations its contract
//! permits, records every call, and is post-install checked against the
//! declared contract for drift.

use std::any::TypeId;
use std::collections::BTreeSet;

use bevy::app::{PostUpdate, PreUpdate, Startup, Update};
use bevy::ecs::message::Message;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::system::{IntoSystem, ReadOnlySystem, ScheduleSystem};
use bevy::prelude::{App, Resource};

use super::command::CommandBus;
use super::phase::Phase;
use super::type_key::TypeKey;

fn assert_declared(
    slot_name: &'static str,
    module_id: &'static str,
    slot: &'static [TypeKey],
    type_id: TypeId,
    type_display: &'static str,
) {
    if !slot.iter().any(|k| k.id == type_id) {
        panic!(
            "install-ctx: module {:?} called {} for {:?} but that type is not in contract.{}",
            module_id, slot_name, type_display, slot_name,
        );
    }
}

fn assert_covered(
    slot_name: &'static str,
    module_id: &'static str,
    declared: &'static [TypeKey],
    seen: &BTreeSet<TypeId>,
) {
    for d in declared {
        if !seen.contains(&d.id) {
            panic!(
                "install-ctx: module {:?} declared contract.{} includes {:?} but install never performed the matching installer call",
                module_id, slot_name, d.name,
            );
        }
    }
}

fn assert_fresh(
    slot_name: &'static str,
    module_id: &'static str,
    type_display: &'static str,
    seen: &BTreeSet<TypeId>,
    type_id: TypeId,
) {
    if seen.contains(&type_id) {
        panic!(
            "install-ctx: module {:?} called {} for {:?} twice in the same install - each declared slot entry must be exercised exactly once",
            module_id, slot_name, type_display,
        );
    }
}

/// Scoped installer for `SimDomain` modules.
///
/// Exposes phase-scoped system scheduling (primary phase for the core logic,
/// `add_command_drain` for command drainage, `add_metric_publish` for metrics),
/// typed resource reads/writes, typed message emission, and typed command
/// consumption. Every call is verified against the module's `SimContract`.
/// After install returns, the registry asserts every declared slot was
/// exercised exactly once and the primary phase received at least one system.
pub struct SimInstaller<'a> {
    app: &'a mut App,
    module_id: &'static str,
    primary_phase: Phase,
    reads: &'static [TypeKey],
    writes: &'static [TypeKey],
    messages_in: &'static [TypeKey],
    messages_out: &'static [TypeKey],
    commands_in: &'static [TypeKey],
    reads_seen: BTreeSet<TypeId>,
    writes_seen: BTreeSet<TypeId>,
    messages_in_seen: BTreeSet<TypeId>,
    messages_out_seen: BTreeSet<TypeId>,
    commands_in_seen: BTreeSet<TypeId>,
    primary_systems_added: usize,
}

impl<'a> SimInstaller<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        app: &'a mut App,
        module_id: &'static str,
        primary_phase: Phase,
        reads: &'static [TypeKey],
        writes: &'static [TypeKey],
        messages_in: &'static [TypeKey],
        messages_out: &'static [TypeKey],
        commands_in: &'static [TypeKey],
    ) -> Self {
        Self {
            app,
            module_id,
            primary_phase,
            reads,
            writes,
            messages_in,
            messages_out,
            commands_in,
            reads_seen: BTreeSet::new(),
            writes_seen: BTreeSet::new(),
            messages_in_seen: BTreeSet::new(),
            messages_out_seen: BTreeSet::new(),
            commands_in_seen: BTreeSet::new(),
            primary_systems_added: 0,
        }
    }

    /// Add a system to `Update` within the module's primary phase.
    pub fn add_system<M>(&mut self, system: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        let phase = self.primary_phase.clone();
        self.app.add_systems(Update, system.in_set(phase));
        self.primary_systems_added += 1;
    }

    /// Add a system that drains a `CommandBus<T>` queue.
    /// Runs in `Phase::Commands`. Use for systems that call `CommandBus<T>::drain(..)`.
    pub fn add_command_drain<M>(&mut self, system: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.app.add_systems(Update, system.in_set(Phase::Commands));
    }

    /// Add a metric-publishing system. Runs in `Phase::Metrics`.
    pub fn add_metric_publish<M>(&mut self, system: impl IntoScheduleConfigs<ScheduleSystem, M>) {
        self.app.add_systems(Update, system.in_set(Phase::Metrics));
    }

    /// Declare intent to read resource `T`. Panics if `T` is not in
    /// `contract.reads`. Tracks coverage.
    pub fn read_resource<T: Resource>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "reads",
            self.module_id,
            self.reads,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "read_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.reads_seen,
            id,
        );
        self.reads_seen.insert(id);
    }

    /// Declare intent to read message `T`. Panics if `T` is not in
    /// `contract.messages_in`. Tracks coverage.
    pub fn read_message<T: Message>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "messages_in",
            self.module_id,
            self.messages_in,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "read_message",
            self.module_id,
            std::any::type_name::<T>(),
            &self.messages_in_seen,
            id,
        );
        self.messages_in_seen.insert(id);
    }

    /// Initialize a resource the module exclusively owns.
    /// Panics if `T` is not listed in `contract.writes` or is already claimed.
    pub fn write_resource<T: Resource + Default>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "write_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.init_resource::<T>();
    }

    /// Insert a non-`Default` resource the module exclusively owns.
    /// Panics if `T` is not listed in `contract.writes` or already claimed.
    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "insert_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.insert_resource(value);
    }

    /// Register a message type this module emits.
    /// Panics if `T` is not listed in `contract.messages_out` or already claimed.
    pub fn emit_message<T: Message>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "messages_out",
            self.module_id,
            self.messages_out,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "emit_message",
            self.module_id,
            std::any::type_name::<T>(),
            &self.messages_out_seen,
            id,
        );
        self.messages_out_seen.insert(id);
        self.app.add_message::<T>();
    }

    /// Register a command payload this module consumes.
    /// Panics if `T` is not listed in `contract.commands_in` or already claimed.
    /// Initializes `CommandBus<T>` if not already present.
    pub fn consume_command<T: Send + Sync + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "commands_in",
            self.module_id,
            self.commands_in,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "consume_command",
            self.module_id,
            std::any::type_name::<T>(),
            &self.commands_in_seen,
            id,
        );
        self.commands_in_seen.insert(id);
        if !self.app.world().contains_resource::<CommandBus<T>>() {
            self.app.init_resource::<CommandBus<T>>();
        }
    }

    pub(crate) fn finalize(self) {
        assert_covered("reads", self.module_id, self.reads, &self.reads_seen);
        assert_covered("writes", self.module_id, self.writes, &self.writes_seen);
        assert_covered(
            "messages_in",
            self.module_id,
            self.messages_in,
            &self.messages_in_seen,
        );
        assert_covered(
            "messages_out",
            self.module_id,
            self.messages_out,
            &self.messages_out_seen,
        );
        assert_covered(
            "commands_in",
            self.module_id,
            self.commands_in,
            &self.commands_in_seen,
        );
        if self.primary_systems_added == 0 {
            panic!(
                "install-ctx: module {:?} declares PRIMARY_PHASE={:?} but install never called ctx.add_system(..) - the primary phase is a fiction",
                self.module_id, self.primary_phase,
            );
        }
    }
}

/// Scoped installer for `StaticData` modules. Systems land in `Startup`.
///
/// Startup systems must be read-only (apart from declared writes/inserts, which
/// go through the installer methods). The `ReadOnlySystem` bound rejects
/// `ResMut<T>`, `EventWriter<T>`, `Commands`, and `&mut World` parameters at
/// compile time.
pub struct DataInstaller<'a> {
    app: &'a mut App,
    module_id: &'static str,
    writes: &'static [TypeKey],
    writes_seen: BTreeSet<TypeId>,
}

impl<'a> DataInstaller<'a> {
    pub(crate) fn new(
        app: &'a mut App,
        module_id: &'static str,
        writes: &'static [TypeKey],
    ) -> Self {
        Self {
            app,
            module_id,
            writes,
            writes_seen: BTreeSet::new(),
        }
    }

    pub fn add_startup_system<S, M>(&mut self, system: S)
    where
        S: IntoSystem<(), (), M> + 'static,
        S::System: ReadOnlySystem,
    {
        self.app.add_systems(Startup, system);
    }

    pub fn write_resource<T: Resource + Default>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "write_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.init_resource::<T>();
    }

    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "insert_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.insert_resource(value);
    }

    pub(crate) fn finalize(self) {
        assert_covered("writes", self.module_id, self.writes, &self.writes_seen);
    }
}

/// Scoped installer for `View` modules. Systems land in `PostUpdate`.
///
/// View systems must be read-only: the `ReadOnlySystem` bound on `add_system`
/// rejects `ResMut<T>`, `EventWriter<T>`, `Commands`, and `&mut World` at
/// compile time. View-private resources declared in `writes` are initialized
/// via `write_resource` / `insert_resource` through the installer, not via
/// system mutation.
pub struct ViewInstaller<'a> {
    app: &'a mut App,
    module_id: &'static str,
    reads: &'static [TypeKey],
    writes: &'static [TypeKey],
    reads_seen: BTreeSet<TypeId>,
    writes_seen: BTreeSet<TypeId>,
}

impl<'a> ViewInstaller<'a> {
    pub(crate) fn new(
        app: &'a mut App,
        module_id: &'static str,
        reads: &'static [TypeKey],
        writes: &'static [TypeKey],
    ) -> Self {
        Self {
            app,
            module_id,
            reads,
            writes,
            reads_seen: BTreeSet::new(),
            writes_seen: BTreeSet::new(),
        }
    }

    pub fn add_system<S, M>(&mut self, system: S)
    where
        S: IntoSystem<(), (), M> + 'static,
        S::System: ReadOnlySystem,
    {
        self.app.add_systems(PostUpdate, system);
    }

    pub fn read_resource<T: Resource>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "reads",
            self.module_id,
            self.reads,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "read_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.reads_seen,
            id,
        );
        self.reads_seen.insert(id);
    }

    pub fn write_resource<T: Resource + Default>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "write_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.init_resource::<T>();
    }

    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "insert_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.insert_resource(value);
    }

    pub(crate) fn finalize(self) {
        assert_covered("reads", self.module_id, self.reads, &self.reads_seen);
        assert_covered("writes", self.module_id, self.writes, &self.writes_seen);
    }
}

/// Scoped installer for `InputUI` modules. Systems land in `PreUpdate`.
///
/// Input systems must be read-only w.r.t. ECS resources; command production
/// goes through `emit_command::<T>()` and `CommandBus<T>`, never direct
/// `ResMut<SimResource>`. The `ReadOnlySystem` bound on `add_system` rejects
/// `ResMut<T>`, `EventWriter<T>`, `Commands`, and `&mut World` at compile time.
pub struct InputInstaller<'a> {
    app: &'a mut App,
    module_id: &'static str,
    reads: &'static [TypeKey],
    writes: &'static [TypeKey],
    commands_out: &'static [TypeKey],
    reads_seen: BTreeSet<TypeId>,
    writes_seen: BTreeSet<TypeId>,
    commands_out_seen: BTreeSet<TypeId>,
}

impl<'a> InputInstaller<'a> {
    pub(crate) fn new(
        app: &'a mut App,
        module_id: &'static str,
        reads: &'static [TypeKey],
        writes: &'static [TypeKey],
        commands_out: &'static [TypeKey],
    ) -> Self {
        Self {
            app,
            module_id,
            reads,
            writes,
            commands_out,
            reads_seen: BTreeSet::new(),
            writes_seen: BTreeSet::new(),
            commands_out_seen: BTreeSet::new(),
        }
    }

    pub fn add_system<S, M>(&mut self, system: S)
    where
        S: IntoSystem<(), (), M> + 'static,
        S::System: ReadOnlySystem,
    {
        self.app.add_systems(PreUpdate, system);
    }

    pub fn read_resource<T: Resource>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "reads",
            self.module_id,
            self.reads,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "read_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.reads_seen,
            id,
        );
        self.reads_seen.insert(id);
    }

    pub fn write_resource<T: Resource + Default>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "write_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.init_resource::<T>();
    }

    pub fn insert_resource<T: Resource>(&mut self, value: T) {
        let id = TypeId::of::<T>();
        assert_declared(
            "writes",
            self.module_id,
            self.writes,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "insert_resource",
            self.module_id,
            std::any::type_name::<T>(),
            &self.writes_seen,
            id,
        );
        self.writes_seen.insert(id);
        self.app.insert_resource(value);
    }

    /// Register a command payload this module produces.
    /// Panics if `T` is not in `contract.commands_out` or already claimed.
    /// Initializes `CommandBus<T>` if not already present.
    pub fn emit_command<T: Send + Sync + 'static>(&mut self) {
        let id = TypeId::of::<T>();
        assert_declared(
            "commands_out",
            self.module_id,
            self.commands_out,
            id,
            std::any::type_name::<T>(),
        );
        assert_fresh(
            "emit_command",
            self.module_id,
            std::any::type_name::<T>(),
            &self.commands_out_seen,
            id,
        );
        self.commands_out_seen.insert(id);
        if !self.app.world().contains_resource::<CommandBus<T>>() {
            self.app.init_resource::<CommandBus<T>>();
        }
    }

    pub(crate) fn finalize(self) {
        assert_covered("reads", self.module_id, self.reads, &self.reads_seen);
        assert_covered("writes", self.module_id, self.writes, &self.writes_seen);
        assert_covered(
            "commands_out",
            self.module_id,
            self.commands_out,
            &self.commands_out_seen,
        );
    }
}
