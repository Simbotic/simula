use simula_behavior::{test::*, BehaviorTrace};

#[test]
fn succeeder_success() {
    let behavior = r#"
    (
        root:(
            Succeeder(()), 
            [
                (Sequence(()),
                [
                    (Debug((message:"Hello, from DebugMessage0!", fail:true))),
                    (Debug((message:"Hello, from DebugMessage1!"))),
                ]),
            ],
        )
    )
    "#;
    let trace = trace_behavior(behavior);
    println!("{:#?}", trace);
    let expected_trace = BehaviorTrace::from_list(&[
        "[0] STARTED simula_behavior::decorators::succeeder::Succeeder",
        "[1] STARTED simula_behavior::composites::sequence::Sequence",
        "[2] STARTED simula_behavior::actions::debug::Debug",
        "[2] FAILURE simula_behavior::actions::debug::Debug",
        "[1] FAILURE simula_behavior::composites::sequence::Sequence",
        "[0] SUCCESS simula_behavior::decorators::succeeder::Succeeder",
    ]);
    assert_eq!(&trace, &expected_trace);
}
