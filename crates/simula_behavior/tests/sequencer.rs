use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn sequence_single_success() {
    let behavior = r#"
    (
        "Sequencer of a few actions",
        Sequencer(()),
        [
            ("Do action 0", Debug((message:"Hello, from DebugMessage0!"))),
            ("Do action 1", Debug((message:"Hello, from DebugMessage1!"))),
            ("Do action 2", Debug((message:"Hello, from DebugMessage2!"))),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Sequencer of a few actions",
        "[2] STARTED Do action 0",
        "[2] SUCCESS Do action 0",
        "[3] STARTED Do action 1",
        "[3] SUCCESS Do action 1",
        "[4] STARTED Do action 2",
        "[4] SUCCESS Do action 2",
        "[1] SUCCESS Sequencer of a few actions",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_single_failure() {
    let behavior = r#"
    (
        "Sequencer of a few actions",
        Sequencer(()), 
        [
            ("Do action 0", Debug((message:"Hello, from DebugMessage0!"))),
            ("Do action 1", Debug((message:"Hello, from DebugMessage1!", fail:true))),
            ("Do action 2", Debug((message:"Hello, from DebugMessage2!"))),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Sequencer of a few actions",
        "[2] STARTED Do action 0",
        "[2] SUCCESS Do action 0",
        "[3] STARTED Do action 1",
        "[3] FAILURE Do action 1",
        "[1] FAILURE Sequencer of a few actions",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_success() {
    let behavior = r#"
    (
        "Sequencer of sequence",
        Sequencer(()),
        [
            (
                "Depth 1 sequence",
                Sequencer(()),
                [
                    (
                        "Depth 2 sequence",
                        Sequencer(()),
                        [
                            ("Deep nested action 1", Debug((message:"Hello, from DebugMessage0!"))),
                        ]
                    ),
                    (
                        "Another depth 2 sequence",
                        Sequencer(()),
                        [
                            ("Deep nested action 2", Debug((message:"Hello, from DebugMessage1!"))),
                            ("Deep nested action 3", Debug((message:"Hello, from DebugMessage2!"))),
                        ]
                    )
                ]
            ),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Sequencer of sequence",
        "[2] STARTED Depth 1 sequence",
        "[3] STARTED Depth 2 sequence",
        "[4] STARTED Deep nested action 1",
        "[4] SUCCESS Deep nested action 1",
        "[3] SUCCESS Depth 2 sequence",
        "[5] STARTED Another depth 2 sequence",
        "[6] STARTED Deep nested action 2",
        "[6] SUCCESS Deep nested action 2",
        "[7] STARTED Deep nested action 3",
        "[7] SUCCESS Deep nested action 3",
        "[5] SUCCESS Another depth 2 sequence",
        "[2] SUCCESS Depth 1 sequence",
        "[1] SUCCESS Sequencer of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_failure() {
    let behavior = r#"
    (
        "Sequencer of sequence",
        Sequencer(()),
        [
            (
                "Nested sequence of sequences",
                Sequencer(()),
                [
                    (
                        "First depth 2 sequence",
                        Sequencer(()),
                        [
                            ("Deep nested action 1", Debug((message:"Hello, from DebugMessage0!"))),
                        ]
                    ),
                    (
                        "Second depth 2 sequence",
                        Sequencer(()),
                        [
                            ("Deep nested action 2", Debug((message:"Hello, from DebugMessage1!", fail:true))),
                            ("Deep nested action 3", Debug((message:"Hello, from DebugMessage2!"))),
                        ]
                    )
                ]
            ),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Sequencer of sequence",
        "[2] STARTED Nested sequence of sequences",
        "[3] STARTED First depth 2 sequence",
        "[4] STARTED Deep nested action 1",
        "[4] SUCCESS Deep nested action 1",
        "[3] SUCCESS First depth 2 sequence",
        "[5] STARTED Second depth 2 sequence",
        "[6] STARTED Deep nested action 2",
        "[6] FAILURE Deep nested action 2",
        "[5] FAILURE Second depth 2 sequence",
        "[2] FAILURE Nested sequence of sequences",
        "[1] FAILURE Sequencer of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}

#[test]
fn sequence_nested_selector_success() {
    let behavior = r#"
    (
        "Sequencer of sequence",
        Sequencer(()),
        [
            (
                "Nested sequence of sequences",
                Sequencer(()),
                [
                    (
                        "First depth 2 sequence",
                        Selector(()),
                        [
                            ("Unlock doors", Debug((message:"Unlocked the doors!"))),
                        ]
                    ),
                    (
                        "Second depth 2 sequence",
                        Sequencer(()),
                        [
                            ("Close doors",Debug((message:"Closed doors!"))),
                            ("Enter door",Debug((message:"Go to selected door!"))),
                        ]
                    )
                ]
            ),
        ]
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Sequencer of sequence",
        "[2] STARTED Nested sequence of sequences",
        "[3] STARTED First depth 2 sequence",
        "[4] STARTED Unlock doors",
        "[4] SUCCESS Unlock doors",
        "[3] SUCCESS First depth 2 sequence",
        "[5] STARTED Second depth 2 sequence",
        "[6] STARTED Close doors",
        "[6] SUCCESS Close doors",
        "[7] STARTED Enter door",
        "[7] SUCCESS Enter door",
        "[5] SUCCESS Second depth 2 sequence",
        "[2] SUCCESS Nested sequence of sequences",
        "[1] SUCCESS Sequencer of sequence",
    ]);
    assert_eq!(&trace, &expected_trace);
}
