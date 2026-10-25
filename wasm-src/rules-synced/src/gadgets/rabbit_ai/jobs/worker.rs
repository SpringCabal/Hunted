use alloc::vec::Vec;
use hunted_common::scare::{self, Scary, Source};
use springcabal::jobs::{Reader, put_f32, put_u32};
pub(crate) fn worker(input: &[u8]) -> Vec<u8> {
    let mut reader = Reader::new(input);
    let count = reader.u32();
    let sources: Vec<_> = (0..count)
        .map(|_| Source {
            x: reader.f32(),
            z: reader.f32(),
            area: scare::Area {
                radius: reader.f32(),
                edge: reader.f32(),
                proximity: reader.f32(),
            },
        })
        .collect();
    let count = reader.u32();
    let mut output = Vec::with_capacity(count as usize * 16);
    for _ in 0..count {
        let light = scare::scariest(&sources, reader.f32(), reader.f32(), reader.f32());
        put_u32(&mut output, u32::from(light.is_some()));
        let value = light.unwrap_or(Scary {
            x: 0.0,
            z: 0.0,
            magnitude: 0.0,
        });
        for v in [value.x, value.z, value.magnitude] {
            put_f32(&mut output, v);
        }
    }
    reader.finish();
    output
}
