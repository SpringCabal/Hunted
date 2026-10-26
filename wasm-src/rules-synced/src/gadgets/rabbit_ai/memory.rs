use alloc::vec::Vec;

use hunted_common::rabbit::RabbitData;

use crate::state::Global;

#[derive(Clone, Copy)]
pub struct RabbitMemory {
    pub unit_id: i32,
    pub last_update: i32,
    pub next_update: i32,
    pub data: RabbitData,
    pub carrying_since: i32,
    pub reserved_carrot: i32,
    pub reserved_eat_time: i32,
    pub eating_progress: i32,
    pub stealing_notified: bool,
    pub carried_carrot: i32,
    pub original_reverse_speed: f32,
}

const NEW_RABBIT_MEMORY: RabbitMemory = RabbitMemory {
    unit_id: -1,
    last_update: 0,
    next_update: 0,
    data: RabbitData::NEW,
    carrying_since: -1,
    reserved_carrot: -1,
    reserved_eat_time: 100,
    eating_progress: 0,
    stealing_notified: false,
    carried_carrot: -1,
    original_reverse_speed: -1.0,
};

#[derive(Default)]
pub struct Rabbits {
    pub memories: Vec<RabbitMemory>,
    slots: Vec<usize>,
    carrot_progress: Vec<i32>,
    reserved_by: Vec<i32>,
}

impl Rabbits {
    pub fn find(&self, rabbit: i32) -> Option<usize> {
        self.slots
            .get(usize::try_from(rabbit).ok()?)
            .copied()
            .filter(|&slot| slot != usize::MAX)
    }

    pub fn memory_index(&mut self, rabbit: i32, frame: i32) -> usize {
        if let Some(index) = self.find(rabbit) {
            return index;
        }
        self.memories.push(RabbitMemory {
            unit_id: rabbit,
            last_update: frame,
            ..NEW_RABBIT_MEMORY
        });
        let index = self.memories.len() - 1;
        let key = usize::try_from(rabbit).expect("nonnegative rabbit ID");
        if key >= self.slots.len() {
            self.slots.resize(key + 1, usize::MAX);
        }
        self.slots[key] = index;
        index
    }

    pub fn retire(&mut self, rabbit: i32) {
        let Some(index) = self.find(rabbit) else {
            return;
        };
        self.release_reservation(index);
        self.slots[rabbit as usize] = usize::MAX;
        self.memories.swap_remove(index);
        if let Some(moved) = self.memories.get(index) {
            self.slots[moved.unit_id as usize] = index;
        }
    }

    pub fn reserve(&mut self, index: usize, carrot: i32) {
        self.release_reservation(index);
        let memory = &mut self.memories[index];
        memory.reserved_carrot = carrot;
        let key = usize::try_from(carrot).expect("nonnegative carrot ID");
        if key >= self.reserved_by.len() {
            self.reserved_by.resize(key + 1, -1);
        }
        self.reserved_by[key] = memory.unit_id;
    }

    pub fn release_reservation(&mut self, index: usize) {
        let memory = &mut self.memories[index];
        if memory.reserved_carrot >= 0
            && self.reserved_by.get(memory.reserved_carrot as usize) == Some(&memory.unit_id)
        {
            self.reserved_by[memory.reserved_carrot as usize] = -1;
        }
        memory.reserved_carrot = -1;
    }

    pub fn reserving(&self, carrot: i32) -> Option<usize> {
        self.reserved_by
            .get(usize::try_from(carrot).ok()?)
            .and_then(|&rabbit| self.find(rabbit))
    }

    pub fn reserved_by_other(&self, rabbit: i32, carrot: i32) -> bool {
        usize::try_from(carrot)
            .ok()
            .and_then(|key| self.reserved_by.get(key))
            .is_some_and(|&owner| owner >= 0 && owner != rabbit)
    }

    pub fn carrot_progress(&self, carrot: i32) -> i32 {
        usize::try_from(carrot)
            .ok()
            .and_then(|key| self.carrot_progress.get(key))
            .copied()
            .unwrap_or(0)
    }

    pub fn set_carrot_progress(&mut self, carrot: i32, progress: i32) {
        let key = usize::try_from(carrot).expect("nonnegative carrot ID");
        if key >= self.carrot_progress.len() {
            self.carrot_progress.resize(key + 1, 0);
        }
        self.carrot_progress[key] = progress.max(0);
    }

    pub fn clear_carrot_progress(&mut self, carrot: i32) {
        if let Some(progress) = usize::try_from(carrot)
            .ok()
            .and_then(|key| self.carrot_progress.get_mut(key))
        {
            *progress = 0;
        }
    }
}

pub struct RabbitCtx<'a> {
    pub rabbits: &'a mut Rabbits,
    pub carrots_stolen: &'a mut i32,
    pub speed_multiplier: f32,
    pub panic_resistance: f32,
}

pub fn with_rabbit_ctx<R>(global: &Global, f: impl FnOnce(&mut RabbitCtx<'_>) -> R) -> R {
    let mut rabbits = global.rabbits.access_mut();
    let mut score = global.score.access_mut();
    let waves = global.waves.access();
    f(&mut RabbitCtx {
        rabbits: &mut rabbits,
        carrots_stolen: &mut score.carrots_stolen,
        speed_multiplier: waves.rabbit_speed_multiplier,
        panic_resistance: waves.rabbit_panic_resistance,
    })
}
