use std::sync::mpsc::Sender;

use crate::flow::FlowCommand;

#[derive(Clone)]
pub struct FlowCommandSender {
    tx: Sender<FlowCommand>,
}

impl FlowCommandSender {
    pub fn new(tx: Sender<FlowCommand>) -> Self {
        Self { tx }
    }

    pub fn send(&self, command: FlowCommand) -> bool {
        self.tx.send(command).is_ok()
    }
}
