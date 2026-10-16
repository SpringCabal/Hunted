use springcabal::defs::DefCache;

#[derive(Clone, Copy)]
pub struct Defs {
    pub rabbit: Option<i32>,
    pub lighthouse: Option<i32>,
}

static DEFS: DefCache<Defs> = DefCache::new(|| Defs {
    rabbit: spring::get_unit_def_id_by_name("rabbit").ok(),
    lighthouse: spring::get_unit_def_id_by_name("lighthouse").ok(),
});

pub fn defs() -> Defs {
    *DEFS.get()
}
