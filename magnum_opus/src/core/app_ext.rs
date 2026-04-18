use bevy::prelude::App;

use super::input_ui::InputUI;
use super::install_ctx::{DataInstaller, InputInstaller, SimInstaller, ViewInstaller};
use super::metrics::MetricsRegistry;
use super::registry::ModuleRegistry;
use super::seal::CoreSeal;
use super::sim_domain::SimDomain;
use super::static_data::StaticData;
use super::view::View;

/// Single entry point for module registration.
///
/// Each method: validates the contract, records the module in `ModuleRegistry`,
/// declares metrics, builds a scoped installer, runs `M::install(&mut ctx)`,
/// then asserts the installer's observations cover every declared slot.
pub trait AppExt {
    fn add_sim<M: SimDomain>(&mut self) -> &mut Self;
    fn add_data<M: StaticData>(&mut self) -> &mut Self;
    fn add_view<M: View>(&mut self) -> &mut Self;
    fn add_input<M: InputUI>(&mut self) -> &mut Self;
    fn finalize_modules(&mut self) -> &mut Self;
}

impl AppExt for App {
    fn add_sim<M: SimDomain>(&mut self) -> &mut Self {
        let contract = M::contract();
        {
            let mut reg = self.world_mut().resource_mut::<ModuleRegistry>();
            reg.register_sim(M::ID, M::PRIMARY_PHASE, &contract);
        }
        {
            let mut mreg = self.world_mut().resource_mut::<MetricsRegistry>();
            for d in contract.metrics {
                mreg.declare(M::ID, *d);
            }
        }
        let mut ctx = SimInstaller::new(
            self,
            M::ID,
            M::PRIMARY_PHASE,
            contract.reads,
            contract.writes,
            contract.messages_in,
            contract.messages_out,
            contract.commands_in,
        );
        M::install(&mut ctx);
        ctx.finalize();
        self
    }

    fn add_data<M: StaticData>(&mut self) -> &mut Self {
        let writes = M::writes();
        let metrics = M::metrics();
        {
            let mut reg = self.world_mut().resource_mut::<ModuleRegistry>();
            reg.register_data(M::ID, writes);
        }
        {
            let mut mreg = self.world_mut().resource_mut::<MetricsRegistry>();
            for d in metrics {
                mreg.declare(M::ID, *d);
            }
        }
        let mut ctx = DataInstaller::new(self, M::ID, writes);
        M::install(&mut ctx);
        ctx.finalize();
        self
    }

    fn add_view<M: View>(&mut self) -> &mut Self {
        let reads = M::reads();
        let writes = M::writes();
        let metrics = M::metrics();
        {
            let mut reg = self.world_mut().resource_mut::<ModuleRegistry>();
            reg.register_view(M::ID, reads, writes);
        }
        {
            let mut mreg = self.world_mut().resource_mut::<MetricsRegistry>();
            for d in metrics {
                mreg.declare(M::ID, *d);
            }
        }
        let mut ctx = ViewInstaller::new(self, M::ID, reads, writes);
        M::install(&mut ctx);
        ctx.finalize();
        self
    }

    fn add_input<M: InputUI>(&mut self) -> &mut Self {
        let reads = M::reads();
        let writes = M::writes();
        let commands_out = M::commands_out();
        let metrics = M::metrics();
        {
            let mut reg = self.world_mut().resource_mut::<ModuleRegistry>();
            reg.register_input(M::ID, reads, writes, commands_out);
        }
        {
            let mut mreg = self.world_mut().resource_mut::<MetricsRegistry>();
            for d in metrics {
                mreg.declare(M::ID, *d);
            }
        }
        let mut ctx = InputInstaller::new(self, M::ID, reads, writes, commands_out);
        M::install(&mut ctx);
        ctx.finalize();
        self
    }

    fn finalize_modules(&mut self) -> &mut Self {
        let result = self
            .world_mut()
            .resource_mut::<ModuleRegistry>()
            .finalize_checks();
        if let Err(errs) = result {
            panic!(
                "module-registry finalize failed:\n  - {}",
                errs.join("\n  - "),
            );
        }
        self.world_mut().resource_mut::<CoreSeal>().set_finalized();
        self
    }
}
