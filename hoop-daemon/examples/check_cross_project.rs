use hoop_daemon::embedding::IndexedItem;
use hoop_daemon::vector_index::VectorIndex;

fn main() {
    let mut index = VectorIndex::new();
    index.rebuild(vec![
        IndexedItem {
            id: "st-1".to_string(),
            project: "project-a".to_string(),
            title: "Implement OAuth2 authentication flow".to_string(),
            kind: "feature".to_string(),
            description: None,
        },
        IndexedItem {
            id: "st-2".to_string(),
            project: "project-b".to_string(),
            title: "Add dark mode".to_string(),
            kind: "task".to_string(),
            description: None,
        },
    ]);

    // Cross-project duplicate should be caught
    let matches = index.check_duplicate(
        "Implement OAuth2 auth flow",
        Some("Set up OAuth2 provider for user login"),
    );
    println!("Matches found: {}", matches.len());
    if matches.is_empty() {
        println!("ERROR: No matches found!");
    } else {
        for m in &matches {
            println!(
                "  Match: {} ({}) - sim: {:.3}",
                m.item.id, m.item.title, m.similarity
            );
        }
    }
}
