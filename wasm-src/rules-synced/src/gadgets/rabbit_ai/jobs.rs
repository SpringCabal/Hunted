mod worker;
use super::*;
use springcabal::jobs::{Reader, put_f32, put_u32};
pub(crate) use worker::worker;

pub(super) struct Prepared {
    pub checking: bool,
    pub position: Position,
    pub light: Option<Scary>,
}

pub(super) fn prepare(
    spring: &spring::Spring<'_, spring::RulesSynced>,
    global: &state::Global,
    rabbits: &[i32],
    surroundings: &Surroundings,
    frame: i32,
) -> Option<Vec<Option<Prepared>>> {
    let mut options = global.jobs.access_mut();
    if !options.enabled(spring) {
        return None;
    }
    let mut positions = Vec::new();
    {
        let memories = global.rabbits.access();
        for (index, &rabbit) in rabbits.iter().enumerate() {
            let memory = memories.find(rabbit).map(|slot| &memories.memories[slot]);
            if memory.is_some_and(|memory| frame < memory.next_update) {
                continue;
            }
            if let Some(position) = units::unit_position(spring, rabbit) {
                positions.push((
                    index,
                    position,
                    memory.map_or(1.0, |memory| carrier_light_factor(memory, frame)),
                ));
            }
        }
    }
    let mut prepared: Vec<_> = (0..rabbits.len()).map(|_| None).collect();
    if positions.is_empty() {
        return Some(prepared);
    }
    // Keep the small light-query reduction in one ordered job packet.
    let chunk_size = positions.len();
    let mut inputs = Vec::new();
    let mut limits = Vec::new();
    for chunk in positions.chunks(chunk_size) {
        let mut input = Vec::new();
        put_u32(&mut input, surroundings.lights.len() as u32);
        for source in &surroundings.lights {
            for v in [
                source.x,
                source.z,
                source.area.radius,
                source.area.edge,
                source.area.proximity,
            ] {
                put_f32(&mut input, v);
            }
        }
        put_u32(&mut input, chunk.len() as u32);
        for (_, p, factor) in chunk {
            for v in [p.x, p.z, *factor] {
                put_f32(&mut input, v);
            }
        }
        limits.push(16 * chunk.len());
        inputs.push(input);
    }
    let output = options.run(
        spring,
        crate::RABBIT_LIGHTS,
        &inputs,
        &limits,
        "rabbit-lights",
        worker,
    )?;
    let mut offset = 0;
    for bytes in output {
        let mut reader = Reader::new(&bytes);
        for _ in 0..chunk_size.min(positions.len() - offset) {
            let present = reader.u32() != 0;
            let light = Scary {
                x: reader.f32(),
                z: reader.f32(),
                magnitude: reader.f32(),
            };
            let (index, position, _) = positions[offset];
            prepared[index] = Some(Prepared {
                checking: options.checking(),
                position,
                light: present.then_some(light),
            });
            offset += 1;
        }
        reader.finish();
    }
    assert_eq!(offset, positions.len());
    Some(prepared)
}
