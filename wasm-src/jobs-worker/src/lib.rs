extern crate alloc;

#[path = "../../rules-synced/src/gadgets/rabbit_ai/jobs/worker.rs"]
mod work;
struct Pure;
impl spring::module::Module<spring::RulesSynced> for Pure {
    fn initialize(_: &mut spring::Spring<'_, spring::RulesSynced>) -> spring::Result<Self> {
        Ok(Self)
    }
}
spring::export_module!(APP, spring::RulesSynced, Pure);
spring::export_jobs!(APP, { RUN => ("rabbit-lights", work::worker) });
