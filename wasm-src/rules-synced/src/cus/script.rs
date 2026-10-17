use alloc::rc::Rc;
use core::cell::RefCell;

use spring::UnitId;

use spring::cus::host::UnitDefScript;
use spring::cus::{
    Angle, Axis, InitCtx, Piece, ScriptCapabilities, TaskDefinition, UnitCtx, UnitScript,
    WreckLevel,
};

use super::carrot::{self, CarriedState, CarrotState};
use super::common::*;
use super::lighthouse::{self, LighthousePieces, LighthouseState};
use super::mine;
use super::rabbit::{self, RabbitPieces, RabbitState};
use super::shotgun::{self, SIG_RELOAD, ShotgunPieces, ShotgunState};
use crate::defs::{self, UnitKind};

pub enum HuntedUnitScript {
    Rabbit(Rc<RefCell<RabbitState>>),
    Carrot(Rc<RefCell<CarrotState>>),
    CarrotCarried(Rc<RefCell<CarriedState>>),
    CarrotDropped { base: Piece },
    Lighthouse(Rc<RefCell<LighthouseState>>),
    Mine,
    Shotgun(Rc<RefCell<ShotgunState>>),
    Inert,
}

impl UnitDefScript for HuntedUnitScript {
    fn capabilities(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
    ) -> Option<ScriptCapabilities> {
        let kind = defs::kind_of(spring, def)?;
        let mut bits = ScriptCapabilities::KILLED | ScriptCapabilities::CREATE;
        if kind == UnitKind::Rabbit {
            bits |= ScriptCapabilities::START_MOVING | ScriptCapabilities::STOP_MOVING;
        }
        Some(ScriptCapabilities::new(bits))
    }

    fn for_def(
        spring: &spring::Spring<'_, spring::RulesSynced>,
        def: i32,
        ctx: &mut InitCtx<'_>,
        _: &mut (),
    ) -> Self {
        match defs::kind_of(spring, def) {
            Some(UnitKind::Rabbit) => {
                let _ = spring
                    .synced_random()
                    .next_int(0, 0)
                    .expect("synced random int");
                Self::Rabbit(Rc::new(RefCell::new(RabbitState {
                    pieces: RabbitPieces::from_init(ctx),
                    moving: None,
                })))
            }
            Some(UnitKind::Carrot) => Self::Carrot(Rc::new(RefCell::new(CarrotState {
                base: piece(ctx, "base"),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }))),
            Some(UnitKind::CarrotCarried) => {
                Self::CarrotCarried(Rc::new(RefCell::new(CarriedState {
                    base: piece(ctx, "base"),
                })))
            }
            Some(UnitKind::CarrotDropped) => Self::CarrotDropped {
                base: piece(ctx, "base"),
            },
            Some(UnitKind::Lighthouse) => {
                Self::Lighthouse(Rc::new(RefCell::new(LighthouseState {
                    pieces: LighthousePieces {
                        light_head: piece(ctx, "LightHead"),
                        light_pitch: piece(ctx, "LightPitch"),
                    },
                    angle: spring
                        .synced_random()
                        .next_float()
                        .expect("synced random float")
                        * TAU,
                })))
            }
            Some(UnitKind::Mine) => Self::Mine,
            Some(UnitKind::Shotgun) => Self::Shotgun(Rc::new(RefCell::new(ShotgunState {
                pieces: ShotgunPieces {
                    shell: piece(ctx, "Shell"),
                    shotgun: piece(ctx, "Shotgun"),
                    reload_anim: piece(ctx, "ReloadAnim"),
                    kickback_rotator: piece(ctx, "KickBackRotator"),
                },
            }))),
            Some(UnitKind::Burrow) | None => Self::Inert,
        }
    }
}

impl UnitScript for HuntedUnitScript {
    fn create(&mut self, ctx: &UnitCtx) {
        match self {
            Self::Rabbit(state) => {
                ctx.spawn(TaskDefinition::with_state(
                    "hunted-rabbit-controller",
                    Rc::clone(state),
                    rabbit::controller,
                ));
            }
            Self::Carrot(state) => {
                let position = unit_position(ctx);
                let ground = ctx
                    .with_spring(|spring| {
                        spring.terrain().get_ground_height(position.x, position.z)
                    })
                    .unwrap_or(position.y);
                let mut carrot = state.borrow_mut();
                carrot.x = position.x;
                carrot.y = ground + 1.0;
                carrot.z = position.z;
                ctx.move_now(carrot.base, Axis::Y, -10.0);
            }
            Self::CarrotCarried(state) => {
                let base = state.borrow().base;
                ctx.turn_now(base, Axis::X, Angle::radians(0.4 * core::f32::consts::PI));
                ctx.move_now(base, Axis::Y, 7.5);
            }
            Self::CarrotDropped { base } => {
                ctx.turn_now(*base, Axis::X, Angle::radians(core::f32::consts::FRAC_PI_2));
                ctx.move_now(*base, Axis::Y, -7.0);
            }
            Self::Lighthouse(state) => lighthouse::create(ctx, Rc::clone(state)),
            Self::Shotgun(state) => {
                let pieces = state.borrow().pieces;
                if pieces.shell != Piece::INVALID {
                    ctx.hide(pieces.shell);
                }
                turn(ctx, pieces.shotgun, Axis::X, 0.0, 0.0);
                turn(ctx, pieces.shotgun, Axis::Y, 0.0, 0.0);
                turn(ctx, pieces.shotgun, Axis::Z, 0.0, 0.0);
            }
            Self::Mine | Self::Inert => {}
        }
    }

    fn start_moving(&mut self, ctx: &UnitCtx, _reversing: bool) {
        if let Self::Rabbit(state) = self {
            rabbit::start_moving(ctx, state);
        }
    }

    fn stop_moving(&mut self, ctx: &UnitCtx) {
        if let Self::Rabbit(state) = self {
            rabbit::stop_moving(ctx, state);
        }
    }

    fn killed(&mut self, ctx: &UnitCtx, _recent_damage: f32, _max_health: f32) -> WreckLevel {
        match self {
            Self::Rabbit(state) => rabbit::killed(ctx, state),
            _ => WreckLevel::None,
        }
    }

    fn call_named(
        &mut self,
        ctx: &UnitCtx,
        name: &str,
        arguments: &[f32],
        returns: &mut [f32],
    ) -> Option<usize> {
        let value = self.named_call(ctx, name, arguments)?;
        match (value, returns.first_mut()) {
            (Some(value), Some(slot)) => {
                *slot = value;
                Some(1)
            }
            _ => Some(0),
        }
    }
}

impl HuntedUnitScript {
    fn named_call(&mut self, ctx: &UnitCtx, name: &str, arguments: &[f32]) -> Option<Option<f32>> {
        let first = arguments.first().copied().unwrap_or(0.0);
        let result = match (&*self, name) {
            (Self::Rabbit(state), "AttachCarrot") => {
                if !arguments.is_empty() {
                    let (ctx, center) = (ctx.clone(), state.borrow().pieces.center);
                    crate::with_addon_context(|addon| {
                        addon
                            .commands()
                            .run(move |_spring| ctx.attach_unit(center, UnitId(first as i32)))
                    });
                }
                None
            }
            (Self::Rabbit(_), "DropCarrot") => {
                if !arguments.is_empty() {
                    let ctx = ctx.clone();
                    crate::with_addon_context(|addon| {
                        addon
                            .commands()
                            .run(move |_spring| ctx.drop_unit(UnitId(first as i32)))
                    });
                }
                None
            }
            (Self::Carrot(state), "StartBeingStolen") => {
                carrot::start_being_stolen(ctx, Rc::clone(state), first);
                Some(1.0)
            }
            (Self::Carrot(state), "StopBeingStolen") => {
                carrot::stop_being_stolen(ctx, state.borrow().base, first);
                Some(1.0)
            }
            (Self::Carrot(_), "ConsumeCarrot") => {
                ctx.spawn(TaskDefinition::new(
                    "hunted-carrot-consume",
                    carrot::consume,
                ));
                Some(1.0)
            }
            (Self::CarrotCarried(state), "BeDropped") => {
                carrot::be_dropped(ctx, Rc::clone(state));
                Some(1.0)
            }
            (Self::CarrotCarried(_), "ScoreCarrot") => {
                ctx.spawn(TaskDefinition::new("hunted-carried-score", carrot::score));
                Some(1.0)
            }
            (Self::CarrotDropped { .. }, "StartBeingStolen" | "StopBeingStolen") => None,
            (Self::CarrotDropped { .. }, "ConsumeCarrot") => {
                ctx.spawn(TaskDefinition::new(
                    "hunted-dropped-consume",
                    carrot::consume,
                ));
                Some(1.0)
            }
            (Self::Mine, "StartFuse") => {
                ctx.spawn(TaskDefinition::new("hunted-mine-fuse", mine::fuse));
                None
            }
            (Self::Shotgun(state), "SetPitch") => {
                let shotgun = state.borrow().pieces.shotgun;
                turn(ctx, shotgun, Axis::X, first, 0.0);
                Some(1.0)
            }
            (Self::Shotgun(state), "Fire") => {
                ctx.signal(SIG_RELOAD);
                ctx.spawn(TaskDefinition::with_state(
                    "hunted-shotgun-fire",
                    Rc::clone(state),
                    shotgun::fire,
                ));
                Some(1.0)
            }
            _ => return None,
        };
        Some(result)
    }
}
