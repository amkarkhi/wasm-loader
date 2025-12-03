// Allow unused public API methods - these are meant for external use
#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents a trace event during WASM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub timestamp: u64,
    pub event_type: TraceEventType,
    pub binary_id: Uuid,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEventType {
    LoadStart,
    LoadComplete,
    LoadError,
    ExecutionStart,
    ExecutionComplete,
    ExecutionError,
    FunctionCall,
    HostFunctionCall,
    MemoryOp,
    FuelCheckpoint,
    PluginLog,
}

/// Execution trace containing all events for a single execution
#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub binary_id: Uuid,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub events: Vec<TraceEvent>,
    pub success: bool,
    pub error_message: Option<String>,
}

impl ExecutionTrace {
    pub fn new(binary_id: Uuid) -> Self {
        Self {
            binary_id,
            start_time: Instant::now(),
            end_time: None,
            events: Vec::new(),
            success: false,
            error_message: None,
        }
    }

    pub fn add_event(
        &mut self,
        event_type: TraceEventType,
        message: String,
        metadata: Option<serde_json::Value>,
    ) {
        let timestamp = self.start_time.elapsed().as_micros() as u64;
        self.events.push(TraceEvent {
            timestamp,
            event_type,
            binary_id: self.binary_id,
            message,
            metadata,
        });
    }

    pub fn complete(&mut self, success: bool, error_message: Option<String>) {
        self.end_time = Some(Instant::now());
        self.success = success;
        self.error_message = error_message;
    }

    pub fn duration(&self) -> Duration {
        match self.end_time {
            Some(end) => end.duration_since(self.start_time),
            None => self.start_time.elapsed(),
        }
    }

    pub fn print(&self) {
        println!("\n=== Execution Trace ===");
        println!("Binary ID: {}", self.binary_id);
        println!("Duration: {:?}", self.duration());
        println!("Success: {}", self.success);
        if let Some(err) = &self.error_message {
            println!("Error: {}", err);
        }
        println!("\nEvents:");
        for event in &self.events {
            println!(
                "  [{:>10}μs] {:?}: {}",
                event.timestamp, event.event_type, event.message
            );
            if let Some(meta) = &event.metadata {
                println!(
                    "    Metadata: {}",
                    serde_json::to_string_pretty(meta).unwrap()
                );
            }
        }
        println!("=====================\n");
    }

    pub fn to_json(&self) -> Result<String> {
        // Convert to a serializable format
        let serializable = serde_json::json!({
            "binary_id": self.binary_id,
            "duration_ms": self.duration().as_millis(),
            "success": self.success,
            "error_message": self.error_message,
            "events": self.events,
        });
        Ok(serde_json::to_string_pretty(&serializable)?)
    }
}

/// Tracer manages execution traces
pub struct Tracer {
    traces: Arc<RwLock<Vec<ExecutionTrace>>>,
    max_traces: usize,
    enabled: bool,
}

impl Tracer {
    pub fn new(enabled: bool, max_traces: usize) -> Self {
        Self {
            traces: Arc::new(RwLock::new(Vec::new())),
            max_traces,
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub async fn start_trace(&self, binary_id: Uuid) -> Option<ExecutionTrace> {
        if !self.enabled {
            return None;
        }
        Some(ExecutionTrace::new(binary_id))
    }

    pub async fn complete_trace(&self, trace: ExecutionTrace) {
        if !self.enabled {
            return;
        }

        let mut traces = self.traces.write().await;

        // Keep only the most recent traces
        if traces.len() >= self.max_traces {
            traces.remove(0);
        }

        traces.push(trace);
    }

    pub async fn get_traces(&self) -> Vec<ExecutionTrace> {
        self.traces.read().await.clone()
    }

    pub async fn get_trace(&self, binary_id: Uuid) -> Option<ExecutionTrace> {
        self.traces
            .read()
            .await
            .iter()
            .rev()
            .find(|t| t.binary_id == binary_id)
            .cloned()
    }

    pub async fn clear_traces(&self) {
        self.traces.write().await.clear();
    }

    pub async fn print_all_traces(&self) {
        let traces = self.traces.read().await;
        println!("\n=== All Execution Traces ({}) ===", traces.len());
        for trace in traces.iter() {
            trace.print();
        }
    }

    pub async fn export_traces(&self) -> Result<String> {
        let traces = self.traces.read().await;
        let serializable: Vec<serde_json::Value> = traces
            .iter()
            .map(|t| {
                serde_json::json!({
                    "binary_id": t.binary_id,
                    "duration_ms": t.duration().as_millis(),
                    "success": t.success,
                    "error_message": t.error_message,
                    "events": t.events,
                })
            })
            .collect();
        Ok(serde_json::to_string_pretty(&serializable)?)
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new(true, 100)
    }
}

impl Clone for Tracer {
    fn clone(&self) -> Self {
        Self {
            traces: Arc::clone(&self.traces),
            max_traces: self.max_traces,
            enabled: self.enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracer() {
        let tracer = Tracer::new(true, 10);
        let binary_id = Uuid::new_v4();

        let mut trace = tracer.start_trace(binary_id).await.unwrap();
        trace.add_event(
            TraceEventType::ExecutionStart,
            "Starting execution".to_string(),
            None,
        );
        trace.add_event(
            TraceEventType::ExecutionComplete,
            "Execution completed".to_string(),
            None,
        );
        trace.complete(true, None);

        tracer.complete_trace(trace).await;

        let traces = tracer.get_traces().await;
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].binary_id, binary_id);
        assert_eq!(traces[0].events.len(), 2);
    }
}
