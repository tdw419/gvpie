//! The core logic for the Learning Chatbot.

use crate::pxos_db::PxosDatabase;

pub struct LearningChatbot;

impl LearningChatbot {
    pub fn process_message(db: &mut PxosDatabase, message: &str) {
        db.conversation_history.push(format!("user: {}", message));
        let quick_analysis = Self::quick_analyze(message);
        db.conversation_history.push(format!("bot: {}", quick_analysis));

        if Self::contains_improvement_ideas(message) {
            db.improvement_queue.push(message.to_string());
            db.conversation_history.push("bot: Found an improvement idea! Added to the queue.".to_string());
        }
    }

    fn quick_analyze(message: &str) -> String {
        if message.contains("slow") {
            "I'll analyze system performance and identify bottlenecks...".to_string()
        } else if message.contains("error") {
            "Let me check recent error patterns and suggest fixes...".to_string()
        } else {
            "I'm processing your input for system improvements...".to_string()
        }
    }

    fn contains_improvement_ideas(message: &str) -> bool {
        let triggers = ["slow", "better", "improve", "problem", "issue", "optimize", "fix"];
        triggers.iter().any(|&trigger| message.contains(trigger))
    }
}
