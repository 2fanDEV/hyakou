use std::sync::mpsc::Sender;

use crate::flow::RendererCommand;

#[derive(Clone)]
pub struct FlowCommandSender {
    tx: Sender<RendererCommand>,
}

impl FlowCommandSender {
    pub fn new(tx: Sender<RendererCommand>) -> Self {
        Self { tx }
    }

    pub fn send(&self, command: RendererCommand) -> bool {
        self.tx.send(command).is_ok()
    }
}
