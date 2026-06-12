//! Shared JSON types for AMUD server ↔ agent communication.

pub mod ipc;
pub mod telemetry;

pub use ipc::{
    agent_auth_proof, AgentAuthMessage, AuthProofMessage, ChallengeMessage, ConfigRequest,
};
pub use telemetry::{AgentTelemetry, LxcContainer, NetworkTelemetry};
