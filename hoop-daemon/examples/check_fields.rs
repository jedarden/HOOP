fn main() {
    let json = r#"{"id":"tr-open-001","content_hash":null,"title":"Fix memory leak in parser","description":"Synthetic test bead","design":"","acceptance_criteria":"","notes":"","status":"open","priority":3,"issue_type":"bug","assignee":null,"owner":"","estimated_minutes":null,"created_at":"2026-05-13T22:53:36Z","created_by":"system","updated_at":"2026-05-13T22:53:36Z","closed_at":null,"close_reason":"","closed_by_session":"","due_at":null,"defer_until":null,"external_ref":null,"source_system":"","source_repo":"","deleted_at":null,"deleted_by":"","delete_reason":"","original_type":"","compaction_level":0,"compacted_at":null,"compacted_at_commit":null,"original_size":null,"sender":"","ephemeral":0,"pinned":0,"is_template":0,"dependencies":[],"schema_version":"1.0.0"}"#;
    
    let v: serde_json::Value = serde_json::from_str(json).unwrap();
    if let Some(obj) = v.as_object() {
        for key in obj.keys() {
            println!("{}", key);
        }
    }
}
