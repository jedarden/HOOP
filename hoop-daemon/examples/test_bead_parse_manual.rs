#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Bead {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub design: String,
    #[serde(default)]
    pub acceptance_criteria: String,
    #[serde(default)]
    pub notes: String,
    pub status: String,
    pub priority: i64,
    #[serde(rename = "issue_type")]
    pub issue_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub schema_version: String,
}

fn main() {
    let json = r#"{"id":"tr-open-001","content_hash":null,"title":"Fix memory leak in parser","description":"Synthetic test bead","design":"","acceptance_criteria":"","notes":"","status":"open","priority":3,"issue_type":"bug","assignee":null,"owner":"","estimated_minutes":null,"created_at":"2026-05-13T22:53:36Z","created_by":"system","updated_at":"2026-05-13T22:53:36Z","closed_at":null,"close_reason":"","closed_by_session":"","due_at":null,"defer_until":null,"external_ref":null,"source_system":"","source_repo":"","deleted_at":null,"deleted_by":"","delete_reason":"","original_type":"","compaction_level":0,"compacted_at":null,"compacted_at_commit":null,"original_size":null,"sender":"","ephemeral":0,"pinned":0,"is_template":0,"dependencies":[],"schema_version":"1.0.0"}"#;

    match serde_json::from_str::<Bead>(json) {
        Ok(bead) => println!("Success: {:?}", bead),
        Err(e) => println!("Error: {}", e),
    }
}
