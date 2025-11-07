//! Processes input events for the Pixel OS.

use crate::pxos_db::PxosDatabase;

pub struct EventProcessor;

impl EventProcessor {
    /// Processes all pending input events.
    pub fn process_events(db: &mut PxosDatabase) {
        for event in db.input_events.drain(..) {
            println!("Processing event: type={}, payload={}", event.event_type, event.payload);
            // In the future, this will modify the database state.
        }
    }
}
