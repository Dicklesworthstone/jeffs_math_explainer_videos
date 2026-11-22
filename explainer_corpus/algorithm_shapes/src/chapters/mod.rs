pub mod ch01;
pub mod ch02;
pub mod ch03;
pub mod ch04;
pub mod ch05;
pub mod ch06;
pub mod ch07;
pub mod ch08;
pub mod ch09;
pub mod ch10;
pub mod ch11;
pub mod ch12;
pub mod ch13;
pub mod ch14;
pub mod ch15;
pub mod draw;

pub fn registry() -> Vec<explainer_kit::Chapter> {
    vec![
        explainer_kit::Chapter {
            key: "01_a_route_you_can_solve_yourself",
            make: |kit| Box::new(ch01::Chapter01 { kit }),
            intensity: 0.4,
        },
        explainer_kit::Chapter {
            key: "02_two_jobs_inside_one_computation",
            make: |kit| Box::new(ch02::Chapter02 { kit }),
            intensity: 0.45,
        },
        explainer_kit::Chapter {
            key: "03_change_the_question_keep_the_map",
            make: |kit| Box::new(ch03::Chapter03 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "04_the_rule_that_lets_us_forget",
            make: |kit| Box::new(ch04::Chapter04 { kit }),
            intensity: 0.4,
        },
        explainer_kit::Chapter {
            key: "05_when_two_answers_share_a_badge",
            make: |kit| Box::new(ch05::Chapter05 { kit }),
            intensity: 0.55,
        },
        explainer_kit::Chapter {
            key: "06_why_the_sweep_agrees_with_every_route",
            make: |kit| Box::new(ch06::Chapter06 { kit }),
            intensity: 0.45,
        },
        explainer_kit::Chapter {
            key: "07_counting_roads_layers_and_a_table",
            make: |kit| Box::new(ch07::Chapter07 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "08_what_changes_when_roads_loop",
            make: |kit| Box::new(ch08::Chapter08 { kit }),
            intensity: 0.55,
        },
        explainer_kit::Chapter {
            key: "09_asking_backward_unfold_then_fold",
            make: |kit| Box::new(ch09::Chapter09 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "10_folding_a_tree",
            make: |kit| Box::new(ch10::Chapter10 { kit }),
            intensity: 0.45,
        },
        explainer_kit::Chapter {
            key: "11_history_and_shared_questions",
            make: |kit| Box::new(ch11::Chapter11 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "12_chunks_and_their_summaries",
            make: |kit| Box::new(ch12::Chapter12 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "13_fusion_without_confusion",
            make: |kit| Box::new(ch13::Chapter13 { kit }),
            intensity: 0.55,
        },
        explainer_kit::Chapter {
            key: "14_the_square_that_keeps_a_promise",
            make: |kit| Box::new(ch14::Chapter14 { kit }),
            intensity: 0.5,
        },
        explainer_kit::Chapter {
            key: "15_when_the_machine_applies",
            make: |kit| Box::new(ch15::Chapter15 { kit }),
            intensity: 0.55,
        },
    ]
}
