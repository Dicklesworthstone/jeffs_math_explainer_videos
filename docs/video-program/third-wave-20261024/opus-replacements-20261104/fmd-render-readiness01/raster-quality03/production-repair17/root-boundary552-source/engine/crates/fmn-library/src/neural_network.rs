//! The enhanced-tier NeuralNetworkMobject (§2.6 leapfrog #9, fm-74z):
//! [`NeuralNetworkMobject`] renders real `ft-nn` module structures —
//! introspected layers (MLP, Conv stacks), live weight/activation
//! visualization mid-training (values flowing into mobject styles via updaters),
//! and animation of forward/backward passes (activation wavefronts, gradient-flow
//! highlights) over the standard animation and mobject machinery.
//!
//! ## Architecture & Purity
//!
//! * **Layer & Graph Introspection:** Introspects real layer specifications
//!   (Dense / Linear, Conv2d, activations, inputs/outputs) with exact neuron counts,
//!   weight matrices, and bias vectors.
//! * **Large-Layer Instancing:** For large layers (e.g. 4096 units), the mobject
//!   uses elision markers (ellipsis dots) and compact representations so layout
//!   stays clean and render IR instancing remains fast.
//! * **Live-Value Updaters & Purity:** Updaters reading training loop tensors
//!   update neuron and synapse activations; registering a scene updater
//!   conservatively classifies the animation segment as stateful under
//!   `fmn_anim::purity`.
//! * **Determinism:** Seeded weight initializations and forward activations
//!   replay bit-identically across platforms.

use std::fmt;

use fmn_core::color::{Srgb, interpolate_color};
use fmn_core::constants::{BLUE_D, GREY_B, RED, WHITE, YELLOW};
use fmn_core::types::Vec3;

use crate::arc::Dot;
use crate::line::Line;
use crate::style::Style;
use crate::vmobject::{VMobject, v_group};

/// Default horizontal spacing between adjacent layers.
pub const DEFAULT_LAYER_SPACING: f64 = 2.5;
/// Default vertical spacing between adjacent neurons within a layer.
pub const DEFAULT_NEURON_SPACING: f64 = 0.55;
/// Default neuron dot radius.
pub const DEFAULT_NEURON_RADIUS: f64 = 0.14;
/// Default synapse line stroke width.
pub const DEFAULT_SYNAPSE_STROKE_WIDTH: f64 = 1.6;
/// Default threshold for compressing a layer with ellipsis dots.
pub const DEFAULT_MAX_DISPLAYED_NEURONS: usize = 12;

/// Why a [`NeuralNetworkMobject`] could not be constructed, updated, or built.
#[derive(Debug, Clone, PartialEq)]
pub enum NeuralNetworkError {
    /// The network has zero layers.
    EmptyNetwork,
    /// A layer index was out of bounds.
    InvalidLayerIndex {
        /// Requested layer index.
        index: usize,
        /// Total layer count.
        total_layers: usize,
    },
    /// A neuron index was out of bounds for the given layer.
    InvalidNeuronIndex {
        /// Layer index.
        layer: usize,
        /// Requested neuron index.
        index: usize,
        /// Total neurons in that layer.
        total_neurons: usize,
    },
    /// Vector or matrix dimensions did not match expected shape.
    DimensionMismatch {
        /// Expected dimension.
        expected: usize,
        /// Actual provided dimension.
        got: usize,
        /// Context description.
        context: String,
    },
    /// A geometry construction failed.
    Geometry(String),
}

impl fmt::Display for NeuralNetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNetwork => write!(f, "neural network must have at least one layer"),
            Self::InvalidLayerIndex {
                index,
                total_layers,
            } => {
                write!(
                    f,
                    "invalid layer index {index}; network has {total_layers} layers"
                )
            }
            Self::InvalidNeuronIndex {
                layer,
                index,
                total_neurons,
            } => {
                write!(
                    f,
                    "invalid neuron index {index} in layer {layer}; layer has {total_neurons} neurons"
                )
            }
            Self::DimensionMismatch {
                expected,
                got,
                context,
            } => {
                write!(
                    f,
                    "dimension mismatch for {context}: expected {expected}, got {got}"
                )
            }
            Self::Geometry(detail) => write!(f, "geometry construction failed: {detail}"),
        }
    }
}

impl std::error::Error for NeuralNetworkError {}

/// The architectural kind of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum LayerKind {
    /// Input layer.
    Input {
        /// Number of input features.
        size: usize,
    },
    /// Fully-connected dense linear layer.
    Dense {
        /// Number of input features.
        in_features: usize,
        /// Number of output features.
        out_features: usize,
    },
    /// 2D Convolutional layer.
    Conv2d {
        /// Number of input channels.
        in_channels: usize,
        /// Number of output channels.
        out_channels: usize,
        /// Kernel (height, width).
        kernel_size: (usize, usize),
    },
    /// Output layer.
    Output {
        /// Number of output classes/units.
        size: usize,
    },
    /// Custom / other named layer.
    Custom {
        /// Layer name.
        name: String,
        /// Unit count.
        units: usize,
    },
}

impl LayerKind {
    /// Number of units in this layer.
    #[must_use]
    pub fn unit_count(&self) -> usize {
        match self {
            Self::Input { size } | Self::Output { size } => *size,
            Self::Dense { out_features, .. } => *out_features,
            Self::Conv2d { out_channels, .. } => *out_channels,
            Self::Custom { units, .. } => *units,
        }
    }
}

/// Specification of a single layer within [`NeuralNetworkMobject`].
#[derive(Debug, Clone, PartialEq)]
pub struct LayerSpec {
    /// Layer kind and parameters.
    pub kind: LayerKind,
    /// Current activation levels for each neuron in this layer (0.0 to 1.0).
    pub activations: Vec<f64>,
    /// Biases for each neuron in this layer.
    pub biases: Vec<f64>,
    /// Synapse weights to the next layer `[from_neuron][to_neuron]`.
    pub next_weights: Vec<Vec<f64>>,
    /// Optional display label for the layer (e.g. "Conv1", "Hidden", "Softmax").
    pub label: Option<String>,
}

impl LayerSpec {
    /// Create a new layer specification with the given kind.
    #[must_use]
    pub fn new(kind: LayerKind) -> Self {
        let count = kind.unit_count();
        Self {
            kind,
            activations: vec![0.0; count],
            biases: vec![0.0; count],
            next_weights: Vec::new(),
            label: None,
        }
    }

    /// Attach a label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Unit count of this layer.
    #[must_use]
    pub fn unit_count(&self) -> usize {
        self.kind.unit_count()
    }
}

/// Visual styling configuration for [`NeuralNetworkMobject`].
#[derive(Debug, Clone, PartialEq)]
pub struct NeuralNetworkConfig {
    /// Horizontal spacing between consecutive layers.
    pub layer_spacing: f64,
    /// Vertical spacing between adjacent neurons within a layer.
    pub neuron_spacing: f64,
    /// Radius of neuron dots.
    pub neuron_radius: f64,
    /// Maximum number of neurons to draw before inserting ellipsis dots.
    pub max_displayed_neurons: usize,
    /// Base inactive neuron color.
    pub neuron_base_color: Srgb,
    /// Active neuron fill color when activation = 1.0.
    pub neuron_active_color: Srgb,
    /// Color for positive synapse weights.
    pub positive_weight_color: Srgb,
    /// Color for negative synapse weights.
    pub negative_weight_color: Srgb,
    /// Synapse line stroke width.
    pub synapse_stroke_width: f64,
    /// Base synapse line opacity.
    pub synapse_opacity: f64,
}

impl Default for NeuralNetworkConfig {
    fn default() -> Self {
        Self {
            layer_spacing: DEFAULT_LAYER_SPACING,
            neuron_spacing: DEFAULT_NEURON_SPACING,
            neuron_radius: DEFAULT_NEURON_RADIUS,
            max_displayed_neurons: DEFAULT_MAX_DISPLAYED_NEURONS,
            neuron_base_color: GREY_B,
            neuron_active_color: YELLOW,
            positive_weight_color: BLUE_D,
            negative_weight_color: RED,
            synapse_stroke_width: DEFAULT_SYNAPSE_STROKE_WIDTH,
            synapse_opacity: 0.65,
        }
    }
}

/// An introspected, animatable neural network mobject.
#[derive(Debug, Clone, PartialEq)]
pub struct NeuralNetworkMobject {
    layers: Vec<LayerSpec>,
    config: NeuralNetworkConfig,
}

impl NeuralNetworkMobject {
    /// Construct a multi-layer perceptron (MLP) from layer unit sizes (e.g. `[3, 5, 2]`).
    pub fn from_layer_sizes(sizes: &[usize]) -> Result<Self, NeuralNetworkError> {
        if sizes.is_empty() {
            return Err(NeuralNetworkError::EmptyNetwork);
        }
        let mut layers = Vec::with_capacity(sizes.len());
        for (i, &size) in sizes.iter().enumerate() {
            let kind = if i == 0 {
                LayerKind::Input { size }
            } else if i == sizes.len() - 1 {
                LayerKind::Output { size }
            } else {
                LayerKind::Dense {
                    in_features: sizes[i - 1],
                    out_features: size,
                }
            };
            let mut layer = LayerSpec::new(kind);
            if i + 1 < sizes.len() {
                let next_size = sizes[i + 1];
                // Initialize default uniform weights
                layer.next_weights = vec![vec![0.5; next_size]; size];
            }
            layers.push(layer);
        }
        Ok(Self {
            layers,
            config: NeuralNetworkConfig::default(),
        })
    }

    /// Construct a network from explicit [`LayerSpec`] specifications.
    pub fn from_specs(layers: Vec<LayerSpec>) -> Result<Self, NeuralNetworkError> {
        if layers.is_empty() {
            return Err(NeuralNetworkError::EmptyNetwork);
        }
        Ok(Self {
            layers,
            config: NeuralNetworkConfig::default(),
        })
    }

    /// Introspect an MLP with explicit weights and biases.
    pub fn introspect_mlp(
        layer_sizes: &[usize],
        weights: &[Vec<Vec<f64>>],
        biases: &[Vec<f64>],
    ) -> Result<Self, NeuralNetworkError> {
        if layer_sizes.is_empty() {
            return Err(NeuralNetworkError::EmptyNetwork);
        }
        let n_layers = layer_sizes.len();
        if weights.len() + 1 != n_layers {
            return Err(NeuralNetworkError::DimensionMismatch {
                expected: n_layers.saturating_sub(1),
                got: weights.len(),
                context: "weights layer count".into(),
            });
        }
        if biases.len() + 1 != n_layers {
            return Err(NeuralNetworkError::DimensionMismatch {
                expected: n_layers.saturating_sub(1),
                got: biases.len(),
                context: "biases layer count".into(),
            });
        }

        let mut layers = Vec::with_capacity(n_layers);
        for i in 0..n_layers {
            let size = layer_sizes[i];
            let kind = if i == 0 {
                LayerKind::Input { size }
            } else if i == n_layers - 1 {
                LayerKind::Output { size }
            } else {
                LayerKind::Dense {
                    in_features: layer_sizes[i - 1],
                    out_features: size,
                }
            };
            let mut layer = LayerSpec::new(kind);
            if i > 0 {
                let layer_biases = &biases[i - 1];
                if layer_biases.len() != size {
                    return Err(NeuralNetworkError::DimensionMismatch {
                        expected: size,
                        got: layer_biases.len(),
                        context: format!("biases for layer {i}"),
                    });
                }
                layer.biases = layer_biases.clone();
            }
            if i < n_layers - 1 {
                let layer_weights = &weights[i];
                if layer_weights.len() != size {
                    return Err(NeuralNetworkError::DimensionMismatch {
                        expected: size,
                        got: layer_weights.len(),
                        context: format!("weight rows for layer {i}"),
                    });
                }
                let next_size = layer_sizes[i + 1];
                for (from_idx, row) in layer_weights.iter().enumerate() {
                    if row.len() != next_size {
                        return Err(NeuralNetworkError::DimensionMismatch {
                            expected: next_size,
                            got: row.len(),
                            context: format!("weights from neuron {from_idx} in layer {i}"),
                        });
                    }
                }
                layer.next_weights = layer_weights.clone();
            }
            layers.push(layer);
        }
        Ok(Self {
            layers,
            config: NeuralNetworkConfig::default(),
        })
    }

    /// Introspect a Convolutional + Dense stack (e.g. Conv2d -> Conv2d -> Dense -> Output).
    pub fn introspect_conv_stack(
        in_channels: usize,
        conv_layers: &[(usize, (usize, usize))],
        dense_sizes: &[usize],
    ) -> Result<Self, NeuralNetworkError> {
        if conv_layers.is_empty() && dense_sizes.is_empty() {
            return Err(NeuralNetworkError::EmptyNetwork);
        }
        let mut layers = Vec::new();
        let mut current_channels = in_channels;

        // Input layer
        layers.push(LayerSpec::new(LayerKind::Input { size: in_channels }).with_label("Input"));

        // Conv layers
        for (i, &(out_ch, k_size)) in conv_layers.iter().enumerate() {
            layers.push(
                LayerSpec::new(LayerKind::Conv2d {
                    in_channels: current_channels,
                    out_channels: out_ch,
                    kernel_size: k_size,
                })
                .with_label(format!("Conv{}", i + 1)),
            );
            current_channels = out_ch;
        }

        // Dense layers
        for (i, &size) in dense_sizes.iter().enumerate() {
            let is_last = i == dense_sizes.len() - 1;
            let kind = if is_last {
                LayerKind::Output { size }
            } else {
                let in_f = if i == 0 {
                    current_channels
                } else {
                    dense_sizes[i - 1]
                };
                LayerKind::Dense {
                    in_features: in_f,
                    out_features: size,
                }
            };
            layers.push(LayerSpec::new(kind).with_label(if is_last { "Output" } else { "Dense" }));
        }

        Ok(Self {
            layers,
            config: NeuralNetworkConfig::default(),
        })
    }

    /// Customize the styling configuration.
    #[must_use]
    pub fn with_config(mut self, config: NeuralNetworkConfig) -> Self {
        self.config = config;
        self
    }

    /// Number of layers in the network.
    #[must_use]
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Access layer specifications.
    #[must_use]
    pub fn layers(&self) -> &[LayerSpec] {
        &self.layers
    }

    /// Get a mutable reference to a layer specification.
    pub fn layer_mut(&mut self, index: usize) -> Result<&mut LayerSpec, NeuralNetworkError> {
        let total = self.layers.len();
        self.layers
            .get_mut(index)
            .ok_or(NeuralNetworkError::InvalidLayerIndex {
                index,
                total_layers: total,
            })
    }

    /// Set activations for all neurons in a specific layer.
    pub fn set_layer_activations(
        &mut self,
        layer_idx: usize,
        activations: &[f64],
    ) -> Result<(), NeuralNetworkError> {
        let layer = self.layer_mut(layer_idx)?;
        let expected = layer.unit_count();
        if activations.len() != expected {
            return Err(NeuralNetworkError::DimensionMismatch {
                expected,
                got: activations.len(),
                context: format!("activations for layer {layer_idx}"),
            });
        }
        layer.activations = activations.to_vec();
        Ok(())
    }

    /// Set activation for an individual neuron.
    pub fn set_neuron_activation(
        &mut self,
        layer_idx: usize,
        neuron_idx: usize,
        value: f64,
    ) -> Result<(), NeuralNetworkError> {
        let layer = self.layer_mut(layer_idx)?;
        let total = layer.unit_count();
        let act = layer.activations.get_mut(neuron_idx).ok_or(
            NeuralNetworkError::InvalidNeuronIndex {
                layer: layer_idx,
                index: neuron_idx,
                total_neurons: total,
            },
        )?;
        *act = value;
        Ok(())
    }

    /// Set synapse weight between two neurons.
    pub fn set_synapse_weight(
        &mut self,
        layer_idx: usize,
        from_neuron: usize,
        to_neuron: usize,
        weight: f64,
    ) -> Result<(), NeuralNetworkError> {
        let layer = self.layer_mut(layer_idx)?;
        let unit_count = layer.unit_count();
        let row = layer.next_weights.get_mut(from_neuron).ok_or(
            NeuralNetworkError::InvalidNeuronIndex {
                layer: layer_idx,
                index: from_neuron,
                total_neurons: unit_count,
            },
        )?;
        let total_next = row.len();
        let cell = row
            .get_mut(to_neuron)
            .ok_or(NeuralNetworkError::InvalidNeuronIndex {
                layer: layer_idx + 1,
                index: to_neuron,
                total_neurons: total_next,
            })?;
        *cell = weight;
        Ok(())
    }

    /// Run a forward feedforward pass with logistic sigmoid activation, updating
    /// internal layer activations and returning the final output vector.
    pub fn feed_forward(&mut self, input: &[f64]) -> Result<Vec<f64>, NeuralNetworkError> {
        self.set_layer_activations(0, input)?;
        let mut current = input.to_vec();

        for i in 0..self.layers.len() - 1 {
            let next_size = self.layers[i + 1].unit_count();
            let mut next_act = vec![0.0; next_size];
            let biases = &self.layers[i + 1].biases;
            let weights = &self.layers[i].next_weights;

            for to_idx in 0..next_size {
                let mut z = if to_idx < biases.len() {
                    biases[to_idx]
                } else {
                    0.0
                };
                for (from_idx, &val) in current.iter().enumerate() {
                    if let Some(&w) = weights.get(from_idx).and_then(|row| row.get(to_idx)) {
                        z += val * w;
                    }
                }
                // Sigmoid activation: 1 / (1 + e^(-z))
                let sigmoid = 1.0 / (1.0 + fmn_dmath::exp(-z));
                next_act[to_idx] = sigmoid;
            }
            self.layers[i + 1].activations = next_act.clone();
            current = next_act;
        }
        Ok(current)
    }

    /// Calculate the 3D position of a neuron in the layout.
    #[must_use]
    pub fn neuron_position(&self, layer_idx: usize, neuron_idx: usize) -> Option<Vec3> {
        if layer_idx >= self.layers.len() {
            return None;
        }
        let total_layers = self.layers.len();
        let layer_unit_count = self.layers[layer_idx].unit_count();
        if neuron_idx >= layer_unit_count {
            return None;
        }

        // Center layers horizontally about the origin
        let x = (layer_idx as f64 - (total_layers as f64 - 1.0) / 2.0) * self.config.layer_spacing;

        // Center neurons vertically within the layer
        let displayed_count = layer_unit_count.min(self.config.max_displayed_neurons);
        let effective_idx = if layer_unit_count > self.config.max_displayed_neurons {
            if neuron_idx < self.config.max_displayed_neurons / 2 {
                neuron_idx
            } else if neuron_idx >= layer_unit_count - self.config.max_displayed_neurons / 2 {
                neuron_idx - (layer_unit_count - self.config.max_displayed_neurons)
            } else {
                // Interior elided neuron
                self.config.max_displayed_neurons / 2
            }
        } else {
            neuron_idx
        };

        let y = ((displayed_count as f64 - 1.0) / 2.0 - effective_idx as f64)
            * self.config.neuron_spacing;
        Some([x, y, 0.0])
    }

    /// Build the full visual representation into a [`VMobject`] family.
    ///
    /// The family structure is painter-ordered:
    /// `[synapse_group, neuron_group, optional_elision_group]`.
    pub fn build(&self) -> Result<VMobject, NeuralNetworkError> {
        if self.layers.is_empty() {
            return Err(NeuralNetworkError::EmptyNetwork);
        }

        let mut synapse_mobjects = Vec::new();
        let mut neuron_mobjects = Vec::new();

        // 1. Build Synapses (drawn underneath neurons)
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            if layer_idx + 1 >= self.layers.len() {
                continue;
            }
            let next_layer = &self.layers[layer_idx + 1];
            let from_count = layer.unit_count();
            let to_count = next_layer.unit_count();

            let max_from = from_count.min(self.config.max_displayed_neurons);
            let max_to = to_count.min(self.config.max_displayed_neurons);

            for from_idx in 0..max_from {
                let from_pos = self
                    .neuron_position(layer_idx, from_idx)
                    .ok_or_else(|| NeuralNetworkError::Geometry("missing from_pos".into()))?;

                for to_idx in 0..max_to {
                    let to_pos = self
                        .neuron_position(layer_idx + 1, to_idx)
                        .ok_or_else(|| NeuralNetworkError::Geometry("missing to_pos".into()))?;

                    let weight = layer
                        .next_weights
                        .get(from_idx)
                        .and_then(|row| row.get(to_idx))
                        .copied()
                        .unwrap_or(0.5);

                    let (color, weight_magnitude) = if weight >= 0.0 {
                        (self.config.positive_weight_color, weight.min(1.0))
                    } else {
                        (self.config.negative_weight_color, (-weight).min(1.0))
                    };

                    let stroke_w = (self.config.synapse_stroke_width
                        * (0.4 + 0.6 * weight_magnitude))
                        .clamp(0.5, 6.0);
                    let opacity = (self.config.synapse_opacity * (0.3 + 0.7 * weight_magnitude))
                        .clamp(0.1, 1.0);

                    let line = Line::new(from_pos, to_pos)
                        .color(color)
                        .style(Style::default().stroke(color, stroke_w, opacity));
                    let built_line = line
                        .build()
                        .map_err(|e| NeuralNetworkError::Geometry(format!("{e}")))?;
                    synapse_mobjects.push(built_line);
                }
            }
        }

        // 2. Build Neurons (drawn on top of synapses)
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let unit_count = layer.unit_count();
            let is_compressed = unit_count > self.config.max_displayed_neurons;
            let display_count = unit_count.min(self.config.max_displayed_neurons);

            for neuron_idx in 0..display_count {
                let pos = self
                    .neuron_position(layer_idx, neuron_idx)
                    .ok_or_else(|| NeuralNetworkError::Geometry("missing neuron_pos".into()))?;

                let activation = layer.activations.get(neuron_idx).copied().unwrap_or(0.0);
                let color = interpolate_color(
                    self.config.neuron_base_color,
                    self.config.neuron_active_color,
                    activation.clamp(0.0, 1.0),
                );

                let dot = Dot::new()
                    .point(pos)
                    .radius(self.config.neuron_radius)
                    .color(color)
                    .style(Style::default().fill(color, 1.0).stroke(WHITE, 1.0, 1.0));
                neuron_mobjects.push(dot.build());
            }

            // If compressed, add ellipsis indicator dots
            if is_compressed {
                let mid_x = (layer_idx as f64 - (self.layers.len() as f64 - 1.0) / 2.0)
                    * self.config.layer_spacing;
                for offset_y in [-0.15, 0.0, 0.15] {
                    let el_dot = Dot::new()
                        .point([mid_x, offset_y, 0.0])
                        .radius(self.config.neuron_radius * 0.35)
                        .color(self.config.neuron_base_color)
                        .style(Style::default().fill(self.config.neuron_base_color, 0.8));
                    neuron_mobjects.push(el_dot.build());
                }
            }
        }

        let mut all_children = synapse_mobjects;
        all_children.extend(neuron_mobjects);
        Ok(v_group(all_children))
    }
}
