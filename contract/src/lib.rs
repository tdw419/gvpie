use serde::{Deserialize, Serialize};

/// Minimal contract data structure so the workspace builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineContract {
    pub name: String,
    pub version: String,
    pub stages: Vec<PipelineStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: String,
    pub description: String,
}

impl PipelineContract {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            stages: Vec::new(),
        }
    }

    pub fn add_stage(mut self, id: impl Into<String>, description: impl Into<String>) -> Self {
        self.stages.push(PipelineStage {
            id: id.into(),
            description: description.into(),
        });
        self
    }
}
