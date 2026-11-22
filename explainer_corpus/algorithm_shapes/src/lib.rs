//! Native production in progress. Only implemented scenes are registered;
//! the full reviewed fifteen-chapter script remains the production scope.
pub mod chapters;
pub mod facts;
pub mod model;
pub const SCRIPT: &str = include_str!("../script.tsv");
