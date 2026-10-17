use spring_addons::AddonContext;
use spring_addons::rules::{Gadget, UnitPreDamagedEvent};

use crate::defs::{self, UnitKind};
use crate::state::Global;

pub struct InvulnerableGadget;

impl Gadget<Global> for InvulnerableGadget {
    fn name(&self) -> &'static str {
        "Invulnerable"
    }

    fn unit_pre_damaged(
        &self,
        _ctx: &AddonContext<'_, Global>,
        event: &UnitPreDamagedEvent,
    ) -> Option<spring::DamageResult> {
        let spring = _ctx.spring();
        let vulnerable = matches!(
            defs::kind_of(spring, event.unit_def_id),
            Some(UnitKind::Rabbit | UnitKind::Lighthouse | UnitKind::Mine)
        );
        (!vulnerable).then(|| spring::DamageResult::unchanged(0.0))
    }
}
