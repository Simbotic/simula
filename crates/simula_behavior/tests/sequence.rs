use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn sequence_single_success() {
    let behavior = r#"
    (
        root:(
            Sequence(()), 
            [
                (Debug((message:"Hello, from DebugMessage0!"))),
                (Debug((message:"Hello, from DebugMessage1!"))),
                (Debug((message:"Hello, from DebugMessage2!"))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::sequence::Sequence",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::actions::debug::Debug",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[0] SUCCESS simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_single_failure() {
    let behavior = r#"
    (
        root:(
            Sequence(()), 
            [
                (Debug((message:"Hello, from DebugMessage0!"))),
                (Debug((message:"Hello, from DebugMessage1!", fail:true))),
                (Debug((message:"Hello, from DebugMessage2!"))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::sequence::Sequence",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::actions::debug::Debug",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[0] FAILURE simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_success() {
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
                                (Debug((message:"Hello, from DebugMessage0!"))),
                            ]
                        ),
                        (
                            Sequence(()),
                            [
                                (Debug((message:"Hello, from DebugMessage1!"))),
                                (Debug((message:"Hello, from DebugMessage2!"))),
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
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[4] STARTED simula_behavior::composites::sequence::Sequence",
        "[5] STARTED simula_behavior::actions::debug::Debug",
        "[5] SUCCESS simula_behavior::actions::debug::Debug",
        "[6] STARTED simula_behavior::actions::debug::Debug",
        "[6] SUCCESS simula_behavior::actions::debug::Debug",
        "[4] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_failure() {
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
                                (Debug((message:"Hello, from DebugMessage0!"))),
                            ]
                        ),
                        (
                            Sequence(()),
                            [
                                (Debug((message:"Hello, from DebugMessage1!", fail:true))),
                                (Debug((message:"Hello, from DebugMessage2!"))),
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
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[4] STARTED simula_behavior::composites::sequence::Sequence",
        "[5] STARTED simula_behavior::actions::debug::Debug",
        "[5] FAILURE simula_behavior::actions::debug::Debug",
        "[4] FAILURE simula_behavior::composites::sequence::Sequence",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] FAILURE simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_selector_success() {
    let behavior = r#"
    (
        root:(
            Sequence(()),
            [
                (
                    Sequence(()),
                    [
                        (
                            Selector(()),
                            [
                                (Debug((message:"Unlocked the doors!"))),
                            ]
                        ),
                        (
                            Sequence(()),
                            [
                                (Debug((message:"Closed doors!"))),
                                (Debug((message:"Go to selected door!"))),
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
        "[2] STARTED simula_behavior::composites::selector::Selector",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::composites::selector::Selector",
        "[4] STARTED simula_behavior::composites::sequence::Sequence",
        "[5] STARTED simula_behavior::actions::debug::Debug",
        "[5] SUCCESS simula_behavior::actions::debug::Debug",
        "[6] STARTED simula_behavior::actions::debug::Debug",
        "[6] SUCCESS simula_behavior::actions::debug::Debug",
        "[4] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[1] SUCCESS simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::composites::sequence::Sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}
