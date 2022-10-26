use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn selector_single_success() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0))),
                (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:0))),
                (DebugAction((message:"Hello, from DebugMessage2!", fail:true, repeat:0))),
            ],
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

#[test]
fn selector_single_failure() {
    let behavior = r#"
    (
        root:(
            Selector(()), 
            [
                (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0))),
                (DebugAction((message:"Hello, from DebugMessage1!", fail:true, repeat:0))),
                (DebugAction((message:"Hello, from DebugMessage2!", fail:true, repeat:0))),
            ],
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
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] FAILURE simula_behavior::actions::debug_action::DebugAction",
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
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:false, repeat:0))),
                ]),
                (Selector(()),
                [
                    (DebugAction((message:"Hello, from DebugMessage2!", fail:false, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage3!", fail:true, repeat:0))),
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
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] SUCCESS simula_behavior::actions::debug_action::DebugAction",
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
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:true, repeat:0))),
                ]),
                (Selector(()),
                [
                    (DebugAction((message:"Hello, from DebugMessage2!", fail:false, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage3!", fail:true, repeat:0))),
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
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[1] FAILURE simula_behavior::composites::selector::Selector",
        "[4] STARTED simula_behavior::composites::selector::Selector",
        "[5] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[5] SUCCESS simula_behavior::actions::debug_action::DebugAction",
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
                    (DebugAction((message:"Hello, from DebugMessage0!", fail:true, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage1!", fail:true, repeat:0))),
                ]),
                (Selector(()),
                [
                    (DebugAction((message:"Hello, from DebugMessage2!", fail:true, repeat:0))),
                    (DebugAction((message:"Hello, from DebugMessage3!", fail:true, repeat:0))),
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
        "[2] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[2] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[3] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[3] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[1] FAILURE simula_behavior::composites::selector::Selector",
        "[4] STARTED simula_behavior::composites::selector::Selector",
        "[5] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[5] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[6] STARTED simula_behavior::actions::debug_action::DebugAction",
        "[6] FAILURE simula_behavior::actions::debug_action::DebugAction",
        "[4] FAILURE simula_behavior::composites::selector::Selector",
        "[0] FAILURE simula_behavior::composites::selector::Selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}