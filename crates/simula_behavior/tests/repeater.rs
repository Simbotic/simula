use bevy::{ecs::system::CommandQueue, prelude::*};
use simula_behavior::{test::*, BehaviorCursor, BehaviorDocument, BehaviorTrace, BehaviorTree};

#[test]
fn repeater_sequence_succeeed() {
    let behavior = r#"
    (
        root:(
            Repeater((repeat:Times(2), repeated:0)),
                [(Sequence(()),[
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:2)),[])
                ])]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[1] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[0] SUCCESS simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}
