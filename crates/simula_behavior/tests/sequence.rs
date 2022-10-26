use bevy::{ecs::system::CommandQueue, prelude::*};
use simula_behavior::{
    actions::*, add_children, composites::*, test::*, BehaviorCursor, BehaviorFailure,
    BehaviorInfo, BehaviorRunning, BehaviorSuccess, BehaviorTrace, BehaviorTree,
};

#[test]
fn sequence_debug_single() {
    let behavior = r#"
    (
        root:(
            Sequence(()), 
            [
                (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                (DebugAction((message:"Hello, from DebugMessage1!", fail:true, repeat:0)),[]),
                (DebugAction((message:"Hello, from DebugMessage2!", fail:false, repeat:0)),[]),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::sequence::Sequence",
        "[1] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[1] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[0] FAILURE simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_debug_nested_success() {
    let behavior = r#"
    (
        root:(
            Sequence(()),
            [
                (
                    Sequence(()),
                    [
                        (
                            Sequence(()),
                            [
                                (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                            ]
                        ),
                        (
                            Sequence(()),
                            [
                                (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:0)),[]),
                                (DebugAction((message:"Hello, from DebugMessage2!", fail:false, repeat:0)),[]),
                            ]
                        )
                    ]
                ),
            ]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::sequence::Sequence",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::composites::sequence::Sequence",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[2] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[4] STARTED simula_behavior::composites::sequence::Sequence",
        "[5] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[5] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[6] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[6] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[4] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_debug_nested_failure() {
    let behavior = r#"
    (
        root:(
            Sequence(()),
            [
                (
                    Sequence(()),
                    [
                        (
                            Sequence(()),
                            [
                                (DebugAction((message:"Hello, from DebugMessage0!", fail:false, repeat:0)),[]),
                            ]
                        ),
                        (
                            Sequence(()),
                            [
                                (DebugAction((message:"Hello, from DebugMessage1!", fail:true, repeat:0)),[]),
                                (DebugAction((message:"Hello, from DebugMessage2!", fail:false, repeat:0)),[]),
                            ]
                        )
                    ]
                ),
            ]
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::sequence::Sequence",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::composites::sequence::Sequence",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] SUCCESS simula_behavior::actions::debug_action::DebugAction",
        "[2] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[4] STARTED simula_behavior::composites::sequence::Sequence",
        "[5] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[5] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[4] FAILURE simula_behavior::composites::sequence::Sequence",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] FAILURE simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}
