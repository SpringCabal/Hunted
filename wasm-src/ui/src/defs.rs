use springcabal::defs::DefCache;

#[derive(Clone, Copy)]
pub struct Defs {
    pub rabbit: Option<i32>,
    pub lighthouse: Option<i32>,
}

static DEFS: DefCache<Defs> = DefCache::new();
pub fn defs(spring: &spring::Spring<'_, spring::Ui>) -> Defs {
    *DEFS.get_or_init(|| Defs {
        rabbit: spring.unit_defs().get_unit_def_id_by_name("rabbit").ok(),
        lighthouse: spring
            .unit_defs()
            .get_unit_def_id_by_name("lighthouse")
            .ok(),
    })
}
