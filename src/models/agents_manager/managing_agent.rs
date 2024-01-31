use crate::models::agents::agent_traits::{FactSheet, SpecialFunctions};
use crate::models::agents_basic::basic_agents::{AgentState, BasicAgent};

#[derive(Debug)]
struct ManagingAgent {
    attributes: FactSheet,
    agents: Vec<Box<dyn SpecialFunctions>>,
}
