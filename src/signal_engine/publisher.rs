// src/signal_engine/publisher.rs

use anyhow::Result;

use crate::persistence::SignalWriter;
use crate::types::signal::Signal;

#[derive(Debug, Clone, Default)]
pub struct SignalPublisher {
    writer: Option<SignalWriter>,
    emitted: Vec<Signal>,
}

impl SignalPublisher {
    pub fn new_memory() -> Self {
        Self {
            writer: None,
            emitted: Vec::new(),
        }
    }

    pub fn with_writer(writer: SignalWriter) -> Self {
        Self {
            writer: Some(writer),
            emitted: Vec::new(),
        }
    }

    pub fn publish(&mut self, signal: Signal) -> Result<()> {
        if let Some(writer) = &self.writer {
            writer.append(&signal)?;
        }

        self.emitted.push(signal);

        Ok(())
    }

    pub fn publish_many<I>(&mut self, signals: I) -> Result<usize>
    where
        I: IntoIterator<Item = Signal>,
    {
        let mut count = 0usize;

        for signal in signals {
            self.publish(signal)?;
            count += 1;
        }

        Ok(count)
    }

    pub fn emitted(&self) -> &[Signal] {
        &self.emitted
    }

    pub fn len(&self) -> usize {
        self.emitted.len()
    }

    pub fn is_empty(&self) -> bool {
        self.emitted.is_empty()
    }

    pub fn clear_memory(&mut self) {
        self.emitted.clear();
    }
}
