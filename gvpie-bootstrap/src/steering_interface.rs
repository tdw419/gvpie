//! The steering interface for the Learning Chatbot.

use crate::pxos_db::PxosDatabase;

pub struct SteeringInterface;

impl SteeringInterface {
    pub fn handle_decision(db: &mut PxosDatabase, proposal_id: usize, decision: &str) {
        if proposal_id >= db.pending_proposals.len() {
            println!("Invalid proposal ID.");
            return;
        }

        let proposal = db.pending_proposals.remove(proposal_id);

        match decision {
            "implement" => {
                println!("Implementing: {}", proposal);
                // In the future, this will apply changes to the system.
            }
            "reject" => {
                println!("Rejecting: {}", proposal);
            }
            _ => {
                println!("Unknown decision: {}", decision);
                // If the decision is unknown, put the proposal back.
                db.pending_proposals.insert(proposal_id, proposal);
            }
        }
    }
}
