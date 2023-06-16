use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn succeeder_success() {
    let behavior = r#"
    (
        "Just succeed",
        Succeeder(()), 
        [
            (
                "Do a few things",
                Sequencer(()),
                [
                    ("Do an action", Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    ("Do another action", Debug((message:"Hello, from DebugMessage1!"))),
                ]
            ),
        ],
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[1] STARTED Just succeed",
        "[2] STARTED Do a few things",
        "[3] STARTED Do an action",
        "[3] FAILURE Do an action",
        "[2] FAILURE Do a few things",
        "[1] SUCCESS Just succeed",
    ]);
    assert_eq!(&trace, &expected_trace);
}
