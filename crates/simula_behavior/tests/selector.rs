use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn selector_single_success() {
    let behavior = r#"
    (
        "Select first successful action",
        Selector(()),
        [
            ("Do action 0", Debug((message:"Hello, from DebugMessage0!", fail:true))),
            ("Do action 1", Debug((message:"Hello, from DebugMessage1!"))),
            ("Do action 2", Debug((message:"Hello, from DebugMessage2!", fail:true))),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Select first successful action",
        "[2] STARTED Do action 0",
        "[2] FAILURE Do action 0",
        "[3] STARTED Do action 1",
        "[3] SUCCESS Do action 1",
        "[1] SUCCESS Select first successful action",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_single_failure() {
    let behavior = r#"
    (
        "Select first successful action",
        Selector(()),
        [
            ("Do action 0", Debug((message:"Hello, from DebugMessage0!", fail:true))),
            ("Do action 1", Debug((message:"Hello, from DebugMessage1!", fail:true))),
            ("Do action 2", Debug((message:"Hello, from DebugMessage2!", fail:true))),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Select first successful action",
        "[2] STARTED Do action 0",
        "[2] FAILURE Do action 0",
        "[3] STARTED Do action 1",
        "[3] FAILURE Do action 1",
        "[4] STARTED Do action 2",
        "[4] FAILURE Do action 2",
        "[1] FAILURE Select first successful action",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_success() {
    let behavior = r#"
    (
        "Select first successful selector",
        Selector(()),
        [
            (
                "Select first successful action",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage1!"))),
                ]
            ),
            (
                "Another select first successful action",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage2!"))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]
            ),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Select first successful selector",
        "[2] STARTED Select first successful action",
        "[3] STARTED Do action 0",
        "[3] FAILURE Do action 0",
        "[4] STARTED Do action 1",
        "[4] SUCCESS Do action 1",
        "[2] SUCCESS Select first successful action",
        "[1] SUCCESS Select first successful selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_fail_first_then_success() {
    let behavior = r#"
    (
        "Select first successful selector",
        Selector(()),
        [
            (
                "Select first successful action",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage1!", fail:true))),
                ]
            ),
            (
                "Another select first successful",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage2!"))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]
            ),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Select first successful selector",
        "[2] STARTED Select first successful action",
        "[3] STARTED Do action 0",
        "[3] FAILURE Do action 0",
        "[4] STARTED Do action 1",
        "[4] FAILURE Do action 1",
        "[2] FAILURE Select first successful action",
        "[5] STARTED Another select first successful",
        "[6] STARTED Do action 0",
        "[6] SUCCESS Do action 0",
        "[5] SUCCESS Another select first successful",
        "[1] SUCCESS Select first successful selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn selector_nested_failure() {
    let behavior = r#"
    (
        "Select first successful selector",
        Selector(()),
        [
            (
                "Select first successful action",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage1!", fail:true))),
                ]
            ),
            (
                "Another select first successful",
                Selector(()),
                [
                    ("Do action 0", Debug((message:"Hello, from DebugMessage2!", fail:true))),
                    ("Do action 1", Debug((message:"Hello, from DebugMessage3!", fail:true))),
                ]
            ),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Select first successful selector",
        "[2] STARTED Select first successful action",
        "[3] STARTED Do action 0",
        "[3] FAILURE Do action 0",
        "[4] STARTED Do action 1",
        "[4] FAILURE Do action 1",
        "[2] FAILURE Select first successful action",
        "[5] STARTED Another select first successful",
        "[6] STARTED Do action 0",
        "[6] FAILURE Do action 0",
        "[7] STARTED Do action 1",
        "[7] FAILURE Do action 1",
        "[5] FAILURE Another select first successful",
        "[1] FAILURE Select first successful selector",
    ]);
    assert_eq!(&trace, &expected_trace);
}
