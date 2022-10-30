use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn selector_single_success() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                (Debug((message:"Hello, from DebugMessage1!"))),
                (Debug((message:"Hello, from DebugMessage2!", fail:true))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::actions::debug::Debug",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] SUCCESS simula_behavior::actions::debug::Debug",
        "[0] SUCCESS simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_single_failure() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                (Debug((message:"Hello, from DebugMessage1!", fail:true))),
                (Debug((message:"Hello, from DebugMessage2!", fail:true))),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::actions::debug::Debug",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] FAILURE simula_behavior::actions::debug::Debug",
        "[0] FAILURE simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_success() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage1!"))),
                ]),
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage2!"))),
                    (Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::composites::selector::Selector",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] SUCCESS simula_behavior::actions::debug::Debug",
        "[1] SUCCESS simula_behavior::composites::selector::Selector",
        "[0] SUCCESS simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_fail_first_then_success() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage1!", fail:true))),
                ]),
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage2!"))),
                    (Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::composites::selector::Selector",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] FAILURE simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::composites::selector::Selector",
        "[4] STARTED simula_behavior::composites::selector::Selector",
        "[5] STARTED simula_behavior::actions::debug::Debug",
        "[5] SUCCESS simula_behavior::actions::debug::Debug",
        "[4] SUCCESS simula_behavior::composites::selector::Selector",
        "[0] SUCCESS simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_failure() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage1!", fail:true))),
                ]),
                (Selector(()),
                [
                    (Debug((message:"Hello, from DebugMessage2!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::composites::selector::Selector",
        "[1] STARTED simula_behavior::composites::selector::Selector",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[3] STARTED simula_behavior::actions::debug::Debug",
        "[3] FAILURE simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::composites::selector::Selector",
        "[4] STARTED simula_behavior::composites::selector::Selector",
        "[5] STARTED simula_behavior::actions::debug::Debug",
        "[5] FAILURE simula_behavior::actions::debug::Debug",
        "[6] STARTED simula_behavior::actions::debug::Debug",
        "[6] FAILURE simula_behavior::actions::debug::Debug",
        "[4] FAILURE simula_behavior::composites::selector::Selector",
        "[0] FAILURE simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}