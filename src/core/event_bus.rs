use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum EngineEvent {
    WindowResized { width: u32, height: u32 },
    StateChanged { from: String, to: String },
    AssetLoaded { path: String },
    Error { message: String },
    Custom { name: String, data: String },
}

pub struct EventBus {
    senders: HashMap<String, Vec<Sender<EngineEvent>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            senders: HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event_type: String) -> Receiver<EngineEvent> {
        let (tx, rx) = unbounded::<EngineEvent>();
        self.senders
            .entry(event_type)
            .or_default()
            .push(tx);
        rx
    }

    pub fn emit(&self, event: EngineEvent) {
        if let Some(senders) = self.senders.get(&format!("{:?}", event)) {
            for tx in senders {
                let _ = tx.send(event.clone());
            }
        }
    }
}
