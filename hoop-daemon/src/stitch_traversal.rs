//! Stitch link traversal service (§4.7)
//!
//! Pure service atop `stitch_links` for traversing relationships between Stitches.
//! Provides:
//! - `parents(id)`: Find all Stitches that link to this Stitch (incoming links)
//! - `children(id)`: Find all Stitches this Stitch links to (outgoing links)
//! - `referenced_by(id)`: Find Stitches that reference this Stitch (kind='references')
//! - `closure(id, kind, depth)`: Recursive traversal with cycle detection
//!
//! Used by:
//! - Stitch view UI (showing parent/child relationships)
//! - Morning Brief (propagating context across spawned chains)
//! - Propagation (diff spreading across reference graphs)
//! - Net-Diff (computing transitive closure of changes)

use anyhow::Result;
use rusqlite::{Connection, params};
use std::collections::{HashSet, VecDeque};
use serde::{Deserialize, Serialize};

#[cfg(test)]
use tempfile::TempDir;

/// A single hop in a traversal path
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StitchLink {
    /// The linked stitch ID
    pub stitch_id: String,
    /// Link kind ('spawned' or 'references')
    pub kind: String,
    /// Source workspace (empty if same workspace)
    pub workspace_from: String,
    /// Target workspace (empty if same workspace)
    pub workspace_to: String,
}

/// A node in a closure traversal with path information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureNode {
    /// The stitch ID
    pub stitch_id: String,
    /// Depth from root (0 = root itself)
    pub depth: u32,
    /// Path from root to this node
    pub path: Vec<String>,
}

/// Traverse incoming links to find parent Stitches.
///
/// Returns all Stitches that have a link pointing to the given Stitch ID.
///
/// # Arguments
/// * `conn` - Database connection
/// * `stitch_id` - Target Stitch ID to find parents of
///
/// # Returns
/// Vector of `StitchLink` representing incoming links
pub fn parents(conn: &Connection, stitch_id: &str) -> Result<Vec<StitchLink>> {
    let mut stmt = conn.prepare(
        "SELECT from_stitch, kind, workspace_from, workspace_to
         FROM stitch_links
         WHERE to_stitch = ?1
         ORDER BY kind, from_stitch"
    )?;

    let links = stmt.query_map(params![stitch_id], |row| {
        Ok(StitchLink {
            stitch_id: row.get(0)?,
            kind: row.get(1)?,
            workspace_from: row.get(2)?,
            workspace_to: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(links)
}

/// Traverse outgoing links to find child Stitches.
///
/// Returns all Stitches that the given Stitch ID links to.
///
/// # Arguments
/// * `conn` - Database connection
/// * `stitch_id` - Source Stitch ID to find children of
///
/// # Returns
/// Vector of `StitchLink` representing outgoing links
pub fn children(conn: &Connection, stitch_id: &str) -> Result<Vec<StitchLink>> {
    let mut stmt = conn.prepare(
        "SELECT to_stitch, kind, workspace_from, workspace_to
         FROM stitch_links
         WHERE from_stitch = ?1
         ORDER BY kind, to_stitch"
    )?;

    let links = stmt.query_map(params![stitch_id], |row| {
        Ok(StitchLink {
            stitch_id: row.get(0)?,
            kind: row.get(1)?,
            workspace_from: row.get(2)?,
            workspace_to: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(links)
}

/// Find Stitches that reference this Stitch.
///
/// Returns incoming links of kind='references' only.
///
/// # Arguments
/// * `conn` - Database connection
/// * `stitch_id` - Target Stitch ID to find references of
///
/// # Returns
/// Vector of `StitchLink` representing reference links
pub fn referenced_by(conn: &Connection, stitch_id: &str) -> Result<Vec<StitchLink>> {
    let mut stmt = conn.prepare(
        "SELECT from_stitch, kind, workspace_from, workspace_to
         FROM stitch_links
         WHERE to_stitch = ?1 AND kind = 'references'
         ORDER BY from_stitch"
    )?;

    let links = stmt.query_map(params![stitch_id], |row| {
        Ok(StitchLink {
            stitch_id: row.get(0)?,
            kind: row.get(1)?,
            workspace_from: row.get(2)?,
            workspace_to: row.get(3)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(links)
}

/// Compute transitive closure from a root Stitch.
///
/// Performs a breadth-first traversal following links of a specific kind,
/// up to a maximum depth. Includes cycle detection to prevent infinite loops.
///
/// # Arguments
/// * `conn` - Database connection
/// * `stitch_id` - Root Stitch ID
/// * `kind` - Link kind to follow ('spawned', 'references', or 'all')
/// * `max_depth` - Maximum traversal depth (0 = root only, None = unlimited)
///
/// # Returns
/// Vector of `ClosureNode` representing all reachable Stitches with path info
///
/// # Examples
/// ```ignore
/// // Find all spawned descendants up to depth 5
/// let nodes = closure(&conn, "stitch-123", "spawned", Some(5))?;
///
/// // Find all descendants (any kind) without depth limit
/// let nodes = closure(&conn, "stitch-123", "all", None)?;
/// ```
pub fn closure(
    conn: &Connection,
    stitch_id: &str,
    kind: &str,
    max_depth: Option<u32>,
) -> Result<Vec<ClosureNode>> {
    let mut result = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u32, Vec<String>)> = VecDeque::new();

    // Start with the root node
    visited.insert(stitch_id.to_string());
    result.push(ClosureNode {
        stitch_id: stitch_id.to_string(),
        depth: 0,
        path: vec![stitch_id.to_string()],
    });
    queue.push_back((stitch_id.to_string(), 0, vec![stitch_id.to_string()]));

    // Build query based on kind filter
    let query = if kind == "all" {
        // Follow all link kinds
        "SELECT to_stitch, kind FROM stitch_links WHERE from_stitch = ?1".to_string()
    } else {
        // Follow specific link kind
        format!(
            "SELECT to_stitch, kind FROM stitch_links WHERE from_stitch = ?1 AND kind = '{}'",
            kind
        )
    };

    let mut stmt = conn.prepare(&query)?;

    while let Some((current_id, current_depth, current_path)) = queue.pop_front() {
        // Check depth limit
        if let Some(max) = max_depth {
            if current_depth >= max {
                continue;
            }
        }

        // Find children
        let child_rows = stmt.query_map(params![&current_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for child_row in child_rows {
            let (child_id, _link_kind) = child_row?;

            // Skip self-links (shouldn't exist due to API validation, but be safe)
            if child_id == current_id {
                continue;
            }

            // Skip already-visited nodes (cycle guard)
            if visited.contains(&child_id) {
                continue;
            }

            visited.insert(child_id.clone());

            // Build path to this node
            let mut child_path = current_path.clone();
            child_path.push(child_id.clone());

            let next_depth = current_depth + 1;
            result.push(ClosureNode {
                stitch_id: child_id.clone(),
                depth: next_depth,
                path: child_path.clone(),
            });

            queue.push_back((child_id, next_depth, child_path));
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> (Connection, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_path = temp_dir.path().join("fleet.db");
        let conn = Connection::open(&db_path).expect("Failed to open fleet.db");

        // Create schema
        conn.execute(
            "CREATE TABLE stitches (
                id TEXT PRIMARY KEY NOT NULL,
                project TEXT NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_activity_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .expect("Failed to create stitches table");

        conn.execute(
            "CREATE TABLE stitch_links (
                from_stitch TEXT NOT NULL,
                to_stitch TEXT NOT NULL,
                kind TEXT NOT NULL CHECK(kind IN ('spawned', 'references')),
                workspace_from TEXT NOT NULL DEFAULT '',
                workspace_to TEXT NOT NULL DEFAULT '',
                PRIMARY KEY (from_stitch, to_stitch, kind),
                FOREIGN KEY (from_stitch) REFERENCES stitches(id) ON DELETE CASCADE,
                FOREIGN KEY (to_stitch) REFERENCES stitches(id) ON DELETE CASCADE
            )",
            [],
        )
        .expect("Failed to create stitch_links table");

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stitch_links_from ON stitch_links(from_stitch)",
            [],
        )
        .expect("Failed to create idx_stitch_links_from");

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stitch_links_to ON stitch_links(to_stitch)",
            [],
        )
        .expect("Failed to create idx_stitch_links_to");

        (conn, temp_dir)
    }

    fn insert_stitch(conn: &Connection, id: &str, project: &str, title: &str) {
        conn.execute(
            "INSERT INTO stitches (id, project, kind, title) VALUES (?1, ?2, 'operator', ?3)",
            params![id, project, title],
        )
        .expect("Failed to insert stitch");
    }

    fn insert_link(
        conn: &Connection,
        from: &str,
        to: &str,
        kind: &str,
        ws_from: &str,
        ws_to: &str,
    ) {
        conn.execute(
            "INSERT INTO stitch_links (from_stitch, to_stitch, kind, workspace_from, workspace_to)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![from, to, kind, ws_from, ws_to],
        )
        .expect("Failed to insert link");
    }

    #[test]
    fn test_parents_single() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "parent", "proj", "Parent");
        insert_stitch(&conn, "child", "proj", "Child");
        insert_link(&conn, "parent", "child", "spawned", "", "");

        let parents = parents(&conn, "child").expect("Failed to get parents");
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].stitch_id, "parent");
        assert_eq!(parents[0].kind, "spawned");
    }

    #[test]
    fn test_parents_multiple() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "p1", "proj", "P1");
        insert_stitch(&conn, "p2", "proj", "P2");
        insert_stitch(&conn, "target", "proj", "Target");

        insert_link(&conn, "p1", "target", "spawned", "", "");
        insert_link(&conn, "p2", "target", "references", "", "");

        let parents = parents(&conn, "target").expect("Failed to get parents");
        assert_eq!(parents.len(), 2);
        assert!(parents.iter().any(|p| p.stitch_id == "p1" && p.kind == "spawned"));
        assert!(parents.iter().any(|p| p.stitch_id == "p2" && p.kind == "references"));
    }

    #[test]
    fn test_children_single() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "parent", "proj", "Parent");
        insert_stitch(&conn, "child", "proj", "Child");
        insert_link(&conn, "parent", "child", "spawned", "", "");

        let children = children(&conn, "parent").expect("Failed to get children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].stitch_id, "child");
        assert_eq!(children[0].kind, "spawned");
    }

    #[test]
    fn test_children_multiple() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "c1", "proj", "C1");
        insert_stitch(&conn, "c2", "proj", "C2");

        insert_link(&conn, "root", "c1", "spawned", "", "");
        insert_link(&conn, "root", "c2", "spawned", "", "");

        let children = children(&conn, "root").expect("Failed to get children");
        assert_eq!(children.len(), 2);
        assert!(children.iter().any(|c| c.stitch_id == "c1"));
        assert!(children.iter().any(|c| c.stitch_id == "c2"));
    }

    #[test]
    fn test_referenced_by() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "ref1", "proj", "Ref1");
        insert_stitch(&conn, "ref2", "proj", "Ref2");
        insert_stitch(&conn, "target", "proj", "Target");
        insert_stitch(&conn, "spawned", "proj", "Spawned");

        insert_link(&conn, "ref1", "target", "references", "", "");
        insert_link(&conn, "ref2", "target", "references", "", "");
        insert_link(&conn, "spawned", "target", "spawned", "", "");

        let refs = referenced_by(&conn, "target").expect("Failed to get references");
        assert_eq!(refs.len(), 2);
        assert!(refs.iter().all(|r| r.kind == "references"));
        assert!(refs.iter().any(|r| r.stitch_id == "ref1"));
        assert!(refs.iter().any(|r| r.stitch_id == "ref2"));
    }

    #[test]
    fn test_closure_linear_chain() {
        let (conn, _temp) = setup_test_db();

        // Create a linear chain: root -> a -> b -> c
        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "a", "proj", "A");
        insert_stitch(&conn, "b", "proj", "B");
        insert_stitch(&conn, "c", "proj", "C");

        insert_link(&conn, "root", "a", "spawned", "", "");
        insert_link(&conn, "a", "b", "spawned", "", "");
        insert_link(&conn, "b", "c", "spawned", "", "");

        let closure_nodes = closure(&conn, "root", "spawned", None)
            .expect("Failed to compute closure");

        assert_eq!(closure_nodes.len(), 4); // root + 3 children

        // Check root
        assert_eq!(closure_nodes[0].stitch_id, "root");
        assert_eq!(closure_nodes[0].depth, 0);

        // Check depths
        let depths: std::collections::HashMap<_, _> = closure_nodes
            .iter()
            .map(|n| (n.stitch_id.as_str(), n.depth))
            .collect();

        assert_eq!(depths["root"], 0);
        assert_eq!(depths["a"], 1);
        assert_eq!(depths["b"], 2);
        assert_eq!(depths["c"], 3);
    }

    #[test]
    fn test_closure_with_depth_limit() {
        let (conn, _temp) = setup_test_db();

        // Create chain: root -> a -> b -> c -> d
        for (id, title) in [("root", "Root"), ("a", "A"), ("b", "B"), ("c", "C"), ("d", "D")] {
            insert_stitch(&conn, id, "proj", title);
        }

        for (from, to) in [("root", "a"), ("a", "b"), ("b", "c"), ("c", "d")] {
            insert_link(&conn, from, to, "spawned", "", "");
        }

        // Limit to depth 2
        let closure_nodes = closure(&conn, "root", "spawned", Some(2))
            .expect("Failed to compute closure");

        assert_eq!(closure_nodes.len(), 3); // root, a, b only

        let ids: Vec<_> = closure_nodes.iter().map(|n| n.stitch_id.as_str()).collect();
        assert!(ids.contains(&"root"));
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
        assert!(!ids.contains(&"c"));
        assert!(!ids.contains(&"d"));
    }

    #[test]
    fn test_closure_with_cycle() {
        let (conn, _temp) = setup_test_db();

        // Create a cycle: a -> b -> c -> a
        insert_stitch(&conn, "a", "proj", "A");
        insert_stitch(&conn, "b", "proj", "B");
        insert_stitch(&conn, "c", "proj", "C");

        insert_link(&conn, "a", "b", "spawned", "", "");
        insert_link(&conn, "b", "c", "spawned", "", "");
        insert_link(&conn, "c", "a", "spawned", "", ""); // cycle back

        // Should not hang, should visit each node once
        let closure_nodes = closure(&conn, "a", "spawned", None)
            .expect("Failed to compute closure");

        assert_eq!(closure_nodes.len(), 3); // Each node visited exactly once

        let ids: std::collections::HashSet<_> =
            closure_nodes.iter().map(|n| n.stitch_id.as_str()).collect();
        assert!(ids.contains("a"));
        assert!(ids.contains("b"));
        assert!(ids.contains("c"));
    }

    #[test]
    fn test_closure_diamond_graph() {
        let (conn, _temp) = setup_test_db();

        // Create diamond: root -> a, root -> b, a -> c, b -> c
        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "a", "proj", "A");
        insert_stitch(&conn, "b", "proj", "B");
        insert_stitch(&conn, "c", "proj", "C");

        insert_link(&conn, "root", "a", "spawned", "", "");
        insert_link(&conn, "root", "b", "spawned", "", "");
        insert_link(&conn, "a", "c", "spawned", "", "");
        insert_link(&conn, "b", "c", "spawned", "", "");

        let closure_nodes = closure(&conn, "root", "spawned", None)
            .expect("Failed to compute closure");

        // c should only appear once (visited via a, skipped via b)
        let ids: Vec<_> = closure_nodes.iter().map(|n| n.stitch_id.as_str()).collect();
        assert_eq!(ids.iter().filter(|&&id| id == "c").count(), 1);
        assert_eq!(closure_nodes.len(), 4); // root, a, b, c
    }

    #[test]
    fn test_closure_disconnected_node() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "child", "proj", "Child");
        insert_stitch(&conn, "orphan", "proj", "Orphan");

        insert_link(&conn, "root", "child", "spawned", "", "");

        let closure_nodes = closure(&conn, "root", "spawned", None)
            .expect("Failed to compute closure");

        assert_eq!(closure_nodes.len(), 2); // root and child only

        let ids: std::collections::HashSet<_> =
            closure_nodes.iter().map(|n| n.stitch_id.as_str()).collect();
        assert!(ids.contains("root"));
        assert!(ids.contains("child"));
        assert!(!ids.contains("orphan"));
    }

    #[test]
    fn test_closure_kind_filter() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "spawned", "proj", "Spawned");
        insert_stitch(&conn, "referenced", "proj", "Referenced");

        insert_link(&conn, "root", "spawned", "spawned", "", "");
        insert_link(&conn, "root", "referenced", "references", "", "");

        // Only spawned
        let spawned = closure(&conn, "root", "spawned", None)
            .expect("Failed to compute closure");
        assert_eq!(spawned.len(), 2);
        let ids: std::collections::HashSet<_> =
            spawned.iter().map(|n| n.stitch_id.as_str()).collect();
        assert!(ids.contains("root"));
        assert!(ids.contains("spawned"));
        assert!(!ids.contains("referenced"));

        // Only references
        let referenced = closure(&conn, "root", "references", None)
            .expect("Failed to compute closure");
        assert_eq!(referenced.len(), 2);
        let ids: std::collections::HashSet<_> =
            referenced.iter().map(|n| n.stitch_id.as_str()).collect();
        assert!(ids.contains("root"));
        assert!(!ids.contains("spawned"));
        assert!(ids.contains("referenced"));

        // All
        let all = closure(&conn, "root", "all", None)
            .expect("Failed to compute closure");
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_closure_fan_out_large() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "root", "proj", "Root");

        // Create 50 children from root
        for i in 0..50 {
            let child_id = format!("child-{}", i);
            insert_stitch(&conn, &child_id, "proj", &format!("Child {}", i));
            insert_link(&conn, "root", &child_id, "spawned", "", "");
        }

        let closure_nodes = closure(&conn, "root", "spawned", None)
            .expect("Failed to compute closure");

        assert_eq!(closure_nodes.len(), 51); // root + 50 children
    }

    #[test]
    fn test_workspace_tracking() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "root", "proj", "Root");
        insert_stitch(&conn, "child", "proj", "Child");

        insert_link(&conn, "root", "child", "spawned", "/ws/a", "/ws/b");

        let children = children(&conn, "root").expect("Failed to get children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].workspace_from, "/ws/a");
        assert_eq!(children[0].workspace_to, "/ws/b");

        let parents = parents(&conn, "child").expect("Failed to get parents");
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].workspace_from, "/ws/a");
        assert_eq!(parents[0].workspace_to, "/ws/b");
    }

    #[test]
    fn test_empty_result() {
        let (conn, _temp) = setup_test_db();

        insert_stitch(&conn, "lonely", "proj", "Lonely");

        assert!(parents(&conn, "lonely").expect("Failed to get parents").is_empty());
        assert!(children(&conn, "lonely").expect("Failed to get children").is_empty());
        assert!(referenced_by(&conn, "lonely")
            .expect("Failed to get referenced_by")
            .is_empty());

        // Closure should return just the root node
        let closure_nodes = closure(&conn, "lonely", "spawned", None)
            .expect("Failed to compute closure");
        assert_eq!(closure_nodes.len(), 1);
        assert_eq!(closure_nodes[0].stitch_id, "lonely");
    }

    #[test]
    fn test_closure_performance_100_stitches() {
        let (conn, _temp) = setup_test_db();

        // Create a tree with 100 nodes
        // root (depth 0)
        //   - 10 children (depth 1)
        //     - 10 grandchildren each (depth 2) = 100 nodes total

        insert_stitch(&conn, "root", "proj", "Root");

        for i in 0..10 {
            let child_id = format!("child-{}", i);
            insert_stitch(&conn, &child_id, "proj", &format!("Child {}", i));
            insert_link(&conn, "root", &child_id, "spawned", "", "");

            for j in 0..9 {
                let grandchild_id = format!("child-{}-{}", i, j);
                insert_stitch(&conn, &grandchild_id, "proj", &format!("Grandchild {}", j));
                insert_link(&conn, &child_id, &grandchild_id, "spawned", "", "");
            }
        }

        // Verify we have 100 stitches
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM stitches", [], |row| row.get(0))
            .expect("Failed to count stitches");
        assert_eq!(count, 100);

        // Performance test: closure to depth 5 should be fast
        let start = std::time::Instant::now();
        let closure_nodes = closure(&conn, "root", "spawned", Some(5))
            .expect("Failed to compute closure");
        let elapsed = start.elapsed();

        // Should find all 100 nodes (depth is only 2, so depth=5 captures everything)
        assert_eq!(closure_nodes.len(), 100);

        // Performance assertion: should complete in under 50ms
        assert!(
            elapsed.as_millis() < 50,
            "Closure took {}ms, expected < 50ms",
            elapsed.as_millis()
        );
    }
}
