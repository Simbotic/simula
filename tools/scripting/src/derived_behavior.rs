use crate::implemented_behavior::ImplementedBehavior;
use bevy::{ecs::system::EntityCommands, prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};
use simula_behavior::prelude::*;
use simula_behavior_macro::BehaviorFactory;

pub struct DerivedBehaviorPlugin;

impl Plugin for DerivedBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(BehaviorTreePlugin::<DerivedBehavior>::default())
            .add_system(subtree::run::<DerivedBehavior>) // Subtrees are typed, need to register them separately
            .register_type::<Subtree<DerivedBehavior>>();
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct DerivedBehaviorAttributes {
    pub pos: Vec2,
}

#[derive(Serialize, Deserialize, TypeUuid, Debug, Clone, Reflect, FromReflect, BehaviorFactory)]
#[uuid = "DBE1D06F-A606-46A2-BFDA-A9480DFEAC6C"]
#[BehaviorAttributes(DerivedBehaviorAttributes)]
pub enum DerivedBehavior {
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
    // Substrees are typed, can load same or different types of subtrees
    Subtree(Subtree<DerivedBehavior>),
    SubImpl(Subtree<ImplementedBehavior>),
}

impl Default for DerivedBehavior {
    fn default() -> Self {
        Self::Debug(Debug::default())
    }
}

impl BehaviorNodeInspectable<DerivedBehavior> for DerivedBehaviorAttributes {
    fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    fn get_pos(&self) -> Vec2 {
        self.pos
    }
}

impl BehaviorInspectable for DerivedBehavior {
    fn color(&self) -> Color {
        match self {
            DerivedBehavior::Debug(_) => Color::hex("#235").unwrap(),
            DerivedBehavior::Selector(_) => Color::hex("#522").unwrap(),
            DerivedBehavior::Sequencer(_) => Color::hex("#252").unwrap(),
            DerivedBehavior::All(_) => Color::hex("#252").unwrap(),
            DerivedBehavior::Any(_) => Color::hex("#522").unwrap(),
            DerivedBehavior::Repeater(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Inverter(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Succeeder(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Wait(_) => Color::hex("#235").unwrap(),
            DerivedBehavior::Delay(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Guard(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Timeout(_) => Color::hex("#440").unwrap(),
            DerivedBehavior::Subtree(_) => Color::hex("#530").unwrap(),
            DerivedBehavior::SubImpl(_) => Color::hex("#530").unwrap(),
        }
    }

    #[rustfmt::skip]
    fn categories(&self) -> Vec<&'static str> {
        match self {
            DerivedBehavior::Debug(_) => vec![<Debug as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Selector(_) => vec![<Selector as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Sequencer(_) => vec![<Sequencer as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::All(_) => vec![<All as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Any(_) => vec![<Any as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Repeater(_) => vec![<Repeater as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Inverter(_) => vec![<Inverter as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Succeeder(_) => vec![<Succeeder as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Wait(_) => vec![<Wait as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Delay(_) => vec![<Delay as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Guard(_) => vec![<Guard as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Timeout(_) => vec![<Timeout as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::Subtree(_) => vec![<Subtree<DerivedBehavior> as BehaviorInfo>::TYPE.as_ref()],
            DerivedBehavior::SubImpl(_) => vec![<Subtree<ImplementedBehavior> as BehaviorInfo>::TYPE.as_ref()],
        }
    }
}
