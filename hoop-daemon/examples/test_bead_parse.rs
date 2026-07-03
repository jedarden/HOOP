use serde::Deserialize;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bead {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub estimated_minutes: Option<u32>,
    pub status: String,
    pub priority: i64,
    #[serde(rename = "issue_type")]
    pub issue_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub design: String,
    #[serde(default)]
    pub acceptance_criteria: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub close_reason: String,
    #[serde(default)]
    pub closed_by_session: String,
    #[serde(default)]
    pub due_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub defer_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub external_ref: Option<String>,
    #[serde(default)]
    pub source_system: String,
    #[serde(default)]
    pub source_repo: String,
    #[serde(default)]
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub deleted_by: String,
    #[serde(default)]
    pub delete_reason: String,
    #[serde(default)]
    pub original_type: String,
    #[serde(default)]
    pub compaction_level: u32,
    #[serde(default)]
    pub compacted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub compacted_at_commit: String,
    #[serde(default)]
    pub original_size: Option<u64>,
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub ephemeral: u32,
    #[serde(default)]
    pub pinned: u32,
    #[serde(default)]
    pub is_template: u32,
}

fn main() {
    let test_json = r#"{"id":"tr-open-001","content_hash":null,"title":"Fix memory leak in parser","description":"Synthetic test bead","design":"","acceptance_criteria":"","notes":"","status":"open","priority":3,"issue_type":"bug","assignee":null,"owner":"","estimated_minutes":null,"created_at":"2026-05-13T22:53:36Z","created_by":"system","updated_at":"2026-05-13T22:53:36Z","closed_at":null,"close_reason":"","closed_by_session":"","due_at":null,"defer_until":null,"external_ref":null,"source_system":"","source_repo":"","deleted_at":null,"deleted_by":"","delete_reason":"","original_type":"","compaction_level":0,"compacted_at":null,"compacted_at_commit":null,"original_size":null,"sender":"","ephemeral":0,"pinned":0,"is_template":0}"#;
    
    match serde_json::from_str::<Bead>(test_json) {
        Ok(bead) => println!("Parsed successfully: {:?}", bead.id),
        Err(e) => println!("Failed to parse: {}", e),
    }
}
