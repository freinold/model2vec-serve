//! model2vec-serve: `OpenAI` and `TEI` compatible embeddings server.

#![warn(missing_docs)]

pub mod auth;
pub mod config;
pub mod errors;
pub mod model;
pub mod routes;
pub mod state;
pub mod telemetry;
