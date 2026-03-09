use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub id: String,
    pub transcript_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}
