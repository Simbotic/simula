use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DebugBehaviorAttributes {
    pub pos: Option<Vec2>,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "7CFA1742-7725-416C-B167-95DA01750E1C"]
pub enum DebugBehavior {
    Debug(Debug),
    Selector(Selector),
    Sequencer(Sequencer),
    All(All),
    Any(Any),
    Repeater(Repeater),
    Inverter(Inverter),
    Succeeder(Succeeder),
    Wait(Wait),
    Delay(Delay),
    Guard(Guard),
    Timeout(Timeout),
    Subtree(Subtree<DebugBehavior>), // Substrees are typed, this loads same tree type
}

impl Default for DebugBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorFactory for DebugBehavior {
    type Attributes = DebugBehaviorAttributes;

    fn insert(&self, commands: &mut EntityCommands) {
        match self {
            DebugBehavior::Debug(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Selector(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Sequencer(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::All(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Any(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Repeater(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Inverter(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Succeeder(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Wait(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Delay(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Guard(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Timeout(data) => BehaviorInfo::insert_with(commands, data),
            DebugBehavior::Subtree(data) => BehaviorInfo::insert_with(commands, data),
        }
    }

    fn label(&self) -> &str {
        match self {
            DebugBehavior::Debug(_) => <Debug as BehaviorInfo>::NAME,
            DebugBehavior::Selector(_) => <Selector as BehaviorInfo>::NAME,
            DebugBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::NAME,
            DebugBehavior::All(_) => <All as BehaviorInfo>::NAME,
            DebugBehavior::Any(_) => <Any as BehaviorInfo>::NAME,
            DebugBehavior::Repeater(_) => <Repeater as BehaviorInfo>::NAME,
            DebugBehavior::Inverter(_) => <Inverter as BehaviorInfo>::NAME,
            DebugBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::NAME,
            DebugBehavior::Wait(_) => <Wait as BehaviorInfo>::NAME,
            DebugBehavior::Delay(_) => <Delay as BehaviorInfo>::NAME,
            DebugBehavior::Guard(_) => <Guard as BehaviorInfo>::NAME,
            DebugBehavior::Timeout(_) => <Timeout as BehaviorInfo>::NAME,
            DebugBehavior::Subtree(_) => <Subtree<DebugBehavior> as BehaviorInfo>::NAME,
        }
    }

    fn reflect(&self) -> &dyn Reflect {
        match self {
            DebugBehavior::Debug(data) => data,
            DebugBehavior::Selector(data) => data,
            DebugBehavior::Sequencer(data) => data,
            DebugBehavior::All(data) => data,
            DebugBehavior::Any(data) => data,
            DebugBehavior::Repeater(data) => data,
            DebugBehavior::Inverter(data) => data,
            DebugBehavior::Succeeder(data) => data,
            DebugBehavior::Wait(data) => data,
            DebugBehavior::Delay(data) => data,
            DebugBehavior::Guard(data) => data,
            DebugBehavior::Timeout(data) => data,
            DebugBehavior::Subtree(data) => data,
        }
    }

    fn reflect_mut(&mut self) -> &mut dyn Reflect {
        match self {
            DebugBehavior::Debug(data) => data,
            DebugBehavior::Selector(data) => data,
            DebugBehavior::Sequencer(data) => data,
            DebugBehavior::All(data) => data,
            DebugBehavior::Any(data) => data,
            DebugBehavior::Repeater(data) => data,
            DebugBehavior::Inverter(data) => data,
            DebugBehavior::Succeeder(data) => data,
            DebugBehavior::Wait(data) => data,
            DebugBehavior::Delay(data) => data,
            DebugBehavior::Guard(data) => data,
            DebugBehavior::Timeout(data) => data,
            DebugBehavior::Subtree(data) => data,
        }
    }

    #[rustfmt::skip]
    fn copy_from(&mut self, entity: Entity, world: &World) -> Result<(), BehaviorMissing> {
        match self {
            DebugBehavior::Debug(data) => *data = world.get::<Debug>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Selector(data) => *data = world.get::<Selector>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Sequencer(data) => *data = world.get::<Sequencer>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::All(data) => *data = world.get::<All>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Any(data) => *data = world.get::<Any>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Repeater(data) => *data = world.get::<Repeater>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Inverter(data) => *data = world.get::<Inverter>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Succeeder(data) => *data = world.get::<Succeeder>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Wait(data) => *data = world.get::<Wait>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Delay(data) => *data = world.get::<Delay>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Guard(data) => *data = world.get::<Guard>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Timeout(data) => *data = world.get::<Timeout>(entity).ok_or(BehaviorMissing)?.clone(),
            DebugBehavior::Subtree(data) => *data = world.get::<Subtree<DebugBehavior>>(entity).ok_or(BehaviorMissing)?.clone(),
        };
        Ok(())
    }

    fn typ(&self) -> BehaviorType {
        match self {
            DebugBehavior::Debug(_) => <Debug as BehaviorInfo>::TYPE,
            DebugBehavior::Selector(_) => <Selector as BehaviorInfo>::TYPE,
            DebugBehavior::Sequencer(_) => <Sequencer as BehaviorInfo>::TYPE,
            DebugBehavior::All(_) => <All as BehaviorInfo>::TYPE,
            DebugBehavior::Any(_) => <Any as BehaviorInfo>::TYPE,
            DebugBehavior::Repeater(_) => <Repeater as BehaviorInfo>::TYPE,
            DebugBehavior::Inverter(_) => <Inverter as BehaviorInfo>::TYPE,
            DebugBehavior::Succeeder(_) => <Succeeder as BehaviorInfo>::TYPE,
            DebugBehavior::Wait(_) => <Wait as BehaviorInfo>::TYPE,
            DebugBehavior::Delay(_) => <Delay as BehaviorInfo>::TYPE,
            DebugBehavior::Guard(_) => <Guard as BehaviorInfo>::TYPE,
            DebugBehavior::Timeout(_) => <Timeout as BehaviorInfo>::TYPE,
            DebugBehavior::Subtree(_) => <Subtree<DebugBehavior> as BehaviorInfo>::TYPE,
        }
    }

    fn color(&self) -> Color {
        match self {
            DebugBehavior::Debug(_) => Color::hex("#235").unwrap(),
            DebugBehavior::Selector(_) => Color::hex("#522").unwrap(),
            DebugBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            DebugBehavior::All(_) => Color::hex("#252").unwrap(),
            DebugBehavior::Any(_) => Color::hex("#522").unwrap(),
            DebugBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Wait(_) => Color::hex("#235").unwrap(),
            DebugBehavior::Delay(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Guard(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Timeout(_) => Color::hex("#440").unwrap(),
            DebugBehavior::Subtree(_) => Color::hex("#440").unwrap(),
        }
    }

    #[rustfmt::skip]
    fn categories(&self) -> Vec<&'static str> {
        match self {
            DebugBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],
            DebugBehavior::Subtree(_) => vec![<Subtree<DebugBehavior> as BehaviorInfo>::TYPE.as_ref()],
        }
    }

    fn list() -> Vec<Self> {
        vec![
            DebugBehavior::Debug(Debug::default()),
            DebugBehavior::Selector(Selector::default()),
            DebugBehavior::Sequencer(Sequencer::default()),
            DebugBehavior::All(All::default()),
            DebugBehavior::Any(Any::default()),
            DebugBehavior::Repeater(Repeater::default()),
            DebugBehavior::Inverter(Inverter::default()),
            DebugBehavior::Succeeder(Succeeder::default()),
            DebugBehavior::Wait(Wait::default()),
            DebugBehavior::Delay(Delay::default()),
            DebugBehavior::Guard(Guard::default()),
            DebugBehavior::Timeout(Timeout::default()),
            DebugBehavior::Subtree(Subtree::<DebugBehavior>::default()),
        ]
    }
}
