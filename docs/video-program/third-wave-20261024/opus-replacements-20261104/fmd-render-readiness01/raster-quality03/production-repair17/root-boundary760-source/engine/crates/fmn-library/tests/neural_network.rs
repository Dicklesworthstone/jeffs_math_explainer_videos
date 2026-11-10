//! Tests for NeuralNetworkMobject (§2.6 leapfrog #9, fm-74z):
//! MLP construction, Conv stack introspection, live-value updaters,
//! purity classification, large-layer elision, and deterministic self-goldens.

use fmn_anim::purity::{ImpureEffect, Purity, classify_wait};
use fmn_hash::sha256;
use fmn_library::neural_network::{LayerKind, NeuralNetworkMobject};
use fmn_mobject::Stage;

fn fail(message: String) -> ! {
    std::panic::panic_any(message)
}

#[test]
fn mlp_construction_and_layer_layout() {
    let net =
        NeuralNetworkMobject::from_layer_sizes(&[3, 4, 2]).unwrap_or_else(|e| fail(format!("{e}")));
    assert_eq!(net.layer_count(), 3);
    assert_eq!(net.layers()[0].unit_count(), 3);
    assert_eq!(net.layers()[1].unit_count(), 4);
    assert_eq!(net.layers()[2].unit_count(), 2);

    // Layer 0 is Input, Layer 1 is Dense, Layer 2 is Output
    assert!(matches!(net.layers()[0].kind, LayerKind::Input { size: 3 }));
    assert!(matches!(
        net.layers()[1].kind,
        LayerKind::Dense {
            in_features: 3,
            out_features: 4
        }
    ));
    assert!(matches!(
        net.layers()[2].kind,
        LayerKind::Output { size: 2 }
    ));

    // Check neuron positions
    let p0_0 = net.neuron_position(0, 0).expect("neuron pos");
    let p0_1 = net.neuron_position(0, 1).expect("neuron pos");
    let p0_2 = net.neuron_position(0, 2).expect("neuron pos");

    // All layer 0 neurons share the same x-coordinate
    assert_eq!(p0_0[0], p0_1[0]);
    assert_eq!(p0_1[0], p0_2[0]);
    // y-coordinates decrease from top to bottom
    assert!(p0_0[1] > p0_1[1]);
    assert!(p0_1[1] > p0_2[1]);

    // Build the visual family
    let built = net.build().unwrap_or_else(|e| fail(format!("{e}")));
    // Synapses: (3*4) + (4*2) = 12 + 8 = 20
    // Neurons: 3 + 4 + 2 = 9
    // Total children = 29
    assert_eq!(built.children().len(), 20 + 9);
}

#[test]
fn conv_stack_introspection() {
    let conv_net = NeuralNetworkMobject::introspect_conv_stack(
        3,                             // 3 input channels (RGB)
        &[(16, (3, 3)), (32, (3, 3))], // Conv1(16), Conv2(32)
        &[64, 10],                     // Dense(64), Output(10)
    )
    .unwrap_or_else(|e| fail(format!("{e}")));

    assert_eq!(conv_net.layer_count(), 5);
    assert_eq!(conv_net.layers()[0].label.as_deref(), Some("Input"));
    assert_eq!(conv_net.layers()[1].label.as_deref(), Some("Conv1"));
    assert_eq!(conv_net.layers()[2].label.as_deref(), Some("Conv2"));
    assert_eq!(conv_net.layers()[3].label.as_deref(), Some("Dense"));
    assert_eq!(conv_net.layers()[4].label.as_deref(), Some("Output"));

    assert_eq!(conv_net.layers()[0].unit_count(), 3);
    assert_eq!(conv_net.layers()[1].unit_count(), 16);
    assert_eq!(conv_net.layers()[2].unit_count(), 32);
    assert_eq!(conv_net.layers()[3].unit_count(), 64);
    assert_eq!(conv_net.layers()[4].unit_count(), 10);
}

#[test]
fn introspect_explicit_weights_and_feed_forward() {
    let layer_sizes = [2, 2, 1];
    let weights = vec![
        vec![vec![1.0, -1.0], vec![-1.0, 1.0]], // Layer 0 -> Layer 1
        vec![vec![2.0], vec![2.0]],             // Layer 1 -> Layer 2
    ];
    let biases = vec![
        vec![0.0, 0.0], // Layer 1 biases
        vec![-1.0],     // Layer 2 bias
    ];

    let mut net = NeuralNetworkMobject::introspect_mlp(&layer_sizes, &weights, &biases)
        .unwrap_or_else(|e| fail(format!("{e}")));

    // Test feed forward on inputs [1.0, 0.0]
    // Layer 1:
    // z0 = 1.0*1.0 + 0.0*(-1.0) + 0.0 = 1.0 -> sigmoid(1.0) ≈ 0.7310585786300049
    // z1 = 1.0*(-1.0) + 0.0*1.0 + 0.0 = -1.0 -> sigmoid(-1.0) ≈ 0.2689414213699951
    // Layer 2:
    // z = 0.7310585786300049 * 2.0 + 0.2689414213699951 * 2.0 - 1.0
    //   = 2.0 * 1.0 - 1.0 = 1.0 -> sigmoid(1.0) ≈ 0.7310585786300049
    let out = net
        .feed_forward(&[1.0, 0.0])
        .unwrap_or_else(|e| fail(format!("{e}")));
    assert_eq!(out.len(), 1);
    let expected = 1.0 / (1.0 + (-1.0_f64).exp());
    assert!((out[0] - expected).abs() < 1e-10);

    // Activations are stored in layers
    assert_eq!(net.layers()[0].activations, vec![1.0, 0.0]);
    assert!((net.layers()[1].activations[0] - expected).abs() < 1e-10);
}

#[test]
fn live_value_updaters_and_purity_classification() {
    let mut stage = Stage::new();
    let net =
        NeuralNetworkMobject::from_layer_sizes(&[2, 3, 1]).unwrap_or_else(|e| fail(format!("{e}")));
    let mob = net.build().unwrap_or_else(|e| fail(format!("{e}")));
    let handle = stage.add(mob);
    stage
        .add_to_scene(handle)
        .unwrap_or_else(|e| fail(format!("{e}")));

    // Initially with no updaters, a play/wait is pure
    let initial_purity = classify_wait(&stage, false);
    assert!(initial_purity.is_pure());

    // Register a training loop updater that modifies the neural network
    stage
        .add_updater(
            handle,
            |_stage, _mob| {
                // live updater simulating values flowing from an ft training tensor
            },
            false,
        )
        .unwrap_or_else(|e| fail(format!("{e}")));

    // Purity classifier now demotes the scene to stateful!
    let updated_purity = classify_wait(&stage, false);
    assert_eq!(
        updated_purity,
        Purity::Stateful(vec![ImpureEffect::SceneUpdater])
    );
}

#[test]
fn large_layer_compression_and_elision() {
    // 4096-neuron layer
    let net = NeuralNetworkMobject::from_layer_sizes(&[4, 4096, 2])
        .unwrap_or_else(|e| fail(format!("{e}")));

    assert_eq!(net.layers()[1].unit_count(), 4096);
    let built = net.build().unwrap_or_else(|e| fail(format!("{e}")));

    // Neurons are capped at max_displayed_neurons (12) + 3 ellipsis dots for the 4096 layer
    // Displayed neurons: 4 (input) + 12 (dense) + 2 (output) = 18
    // Ellipsis dots: 3
    // Synapses: 4 * 12 + 12 * 2 = 48 + 24 = 72
    // Total children = 72 + 18 + 3 = 93
    assert_eq!(built.children().len(), 93);
}

#[test]
fn self_golden_locks_canonical_mlp_build() {
    let mut net =
        NeuralNetworkMobject::from_layer_sizes(&[2, 3, 1]).unwrap_or_else(|e| fail(format!("{e}")));

    // Set deterministic activations and weights
    net.set_layer_activations(0, &[0.8, 0.2])
        .unwrap_or_else(|e| fail(format!("{e}")));
    net.set_layer_activations(1, &[0.5, 0.9, 0.1])
        .unwrap_or_else(|e| fail(format!("{e}")));
    net.set_layer_activations(2, &[0.95])
        .unwrap_or_else(|e| fail(format!("{e}")));

    let built = net.build().unwrap_or_else(|e| fail(format!("{e}")));

    let mut bytes = Vec::new();
    let mut stack = vec![&built];
    while let Some(current) = stack.pop() {
        for point in current.points() {
            bytes.extend_from_slice(&point[0].to_bits().to_le_bytes());
            bytes.extend_from_slice(&point[1].to_bits().to_le_bytes());
            bytes.extend_from_slice(&point[2].to_bits().to_le_bytes());
        }
        for child in current.children() {
            stack.push(child);
        }
    }

    assert_eq!(
        sha256(&bytes).to_hex(),
        "7b57eef56d27a805df6aebda3db30537e29a2504001b6128c7dd88e3622ff22f",
        "canonical neural network build drifted"
    );
}
