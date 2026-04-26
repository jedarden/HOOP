//! Fix patterns service
//!
//! The fix_patterns table stores reusable fix templates for common code issues.
//! Each pattern includes a signature vector for matching, keywords for search,
//! a markdown fix template, and example source stitches where the pattern applies.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::fleet;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A fix pattern for reusable code fix templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPattern {
    pub id: String,
    pub name: String,
    /// Vector of floating-point values representing the pattern signature
    /// Used for similarity matching against issue signatures
    pub signature_vector: Vec<f32>,
    /// Comma-separated keywords for text search
    pub keywords: String,
    /// Markdown template for the recommended fix
    pub recommended_fix_template_md: String,
    /// JSON array of example stitch IDs where this pattern applies
    pub example_source_stitches: Vec<String>,
    /// ISO 8601 timestamp of pattern creation
    pub created_at: String,
    /// Number of times this pattern has been applied
    pub applied_count: i64,
}

/// Pattern match result with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern: FixPattern,
    /// Similarity score (0.0 to 1.0, higher is better)
    pub similarity: f32,
}

/// Create pattern request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatternRequest {
    pub name: String,
    pub signature_vector: Vec<f32>,
    pub keywords: String,
    pub recommended_fix_template_md: String,
    pub example_source_stitches: Vec<String>,
}

/// Update pattern request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatternRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_vector: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recommended_fix_template_md: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example_source_stitches: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Service
// ---------------------------------------------------------------------------

pub struct FixPatternService;

impl FixPatternService {
    /// Create a new fix pattern
    pub fn create(req: &CreatePatternRequest) -> Result<String> {
        let db_path = fleet::db_path();
        let mut conn = Connection::open(&db_path)?;

        let id = uuid::Uuid::new_v4().to_string();

        let signature_json = serde_json::to_string(&req.signature_vector)?;
        let examples_json = serde_json::to_string(&req.example_source_stitches)?;

        conn.execute(
            r#"
            INSERT INTO fix_patterns (
                id, name, signature_vector_json, keywords,
                recommended_fix_template_md, example_source_stitches_json,
                created_at, applied_count
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'), 0)
            "#,
            params![
                &id,
                &req.name,
                &signature_json,
                &req.keywords,
                &req.recommended_fix_template_md,
                &examples_json,
            ],
        )?;

        Ok(id)
    }

    /// Get a pattern by ID
    pub fn get(id: &str) -> Result<Option<FixPattern>> {
        let db_path = fleet::db_path();
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let result = conn.query_row(
            "SELECT id, name, signature_vector_json, keywords,
                    recommended_fix_template_md, example_source_stitches_json,
                    created_at, applied_count
             FROM fix_patterns WHERE id = ?1",
            params![id],
            |row| {
                let signature_json: String = row.get(2)?;
                let signature_vector: Vec<f32> = serde_json::from_str(&signature_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e) as Box<dyn std::error::Error + Send + Sync>))?;

                let examples_json: String = row.get(5)?;
                let example_source_stitches: Vec<String> = serde_json::from_str(&examples_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e) as Box<dyn std::error::Error + Send + Sync>))?;

                Ok(FixPattern {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    signature_vector,
                    keywords: row.get(3)?,
                    recommended_fix_template_md: row.get(4)?,
                    example_source_stitches,
                    created_at: row.get(6)?,
                    applied_count: row.get(7)?,
                })
            },
        );

        match result {
            Ok(pattern) => Ok(Some(pattern)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// List all patterns
    pub fn list() -> Result<Vec<FixPattern>> {
        let db_path = fleet::db_path();
        let conn = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, name, signature_vector_json, keywords,
                    recommended_fix_template_md, example_source_stitches_json,
                    created_at, applied_count
             FROM fix_patterns ORDER BY created_at DESC",
        )?;

        let patterns = stmt.query_map([], |row| {
            let signature_json: String = row.get(2)?;
            let signature_vector: Vec<f32> = serde_json::from_str(&signature_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e) as Box<dyn std::error::Error + Send + Sync>))?;

            let examples_json: String = row.get(5)?;
            let example_source_stitches: Vec<String> = serde_json::from_str(&examples_json)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e) as Box<dyn std::error::Error + Send + Sync>))?;

            Ok(FixPattern {
                id: row.get(0)?,
                name: row.get(1)?,
                signature_vector,
                keywords: row.get(3)?,
                recommended_fix_template_md: row.get(4)?,
                example_source_stitches,
                created_at: row.get(6)?,
                applied_count: row.get(7)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    /// Update an existing pattern
    pub fn update(req: &UpdatePatternRequest) -> Result<()> {
        let db_path = fleet::db_path();
        let conn = Connection::open(&db_path)?;

        // Build dynamic update query based on provided fields
        let mut set_clauses = Vec::new();
        let mut values: Vec<String> = Vec::new();

        if let Some(ref name) = req.name {
            set_clauses.push("name = ?");
            values.push(name.clone());
        }
        if let Some(ref sig) = req.signature_vector {
            set_clauses.push("signature_vector_json = ?");
            values.push(serde_json::to_string(sig)?);
        }
        if let Some(ref keywords) = req.keywords {
            set_clauses.push("keywords = ?");
            values.push(keywords.clone());
        }
        if let Some(ref template) = req.recommended_fix_template_md {
            set_clauses.push("recommended_fix_template_md = ?");
            values.push(template.clone());
        }
        if let Some(ref examples) = req.example_source_stitches {
            set_clauses.push("example_source_stitches_json = ?");
            values.push(serde_json::to_string(examples)?);
        }

        if set_clauses.is_empty() {
            return Ok(());
        }

        values.push(req.id.clone());
        let query = format!(
            "UPDATE fix_patterns SET {} WHERE id = ?",
            set_clauses.join(", ")
        );

        let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
        let affected = conn.execute(&query, params.as_slice())?;
        if affected == 0 {
            anyhow::bail!("Pattern '{}' not found", req.id);
        }

        Ok(())
    }

    /// Delete a pattern
    pub fn delete(id: &str) -> Result<()> {
        let db_path = fleet::db_path();
        let mut conn = Connection::open(&db_path)?;

        let affected = conn.execute("DELETE FROM fix_patterns WHERE id = ?1", params![id])?;
        if affected == 0 {
            anyhow::bail!("Pattern '{}' not found", id);
        }

        Ok(())
    }

    /// Increment the applied_count for a pattern
    pub fn record_application(id: &str) -> Result<()> {
        let db_path = fleet::db_path();
        let mut conn = Connection::open(&db_path)?;

        conn.execute(
            "UPDATE fix_patterns SET applied_count = applied_count + 1 WHERE id = ?1",
            params![id],
        )?;

        Ok(())
    }

    /// Find matching patterns by signature vector using cosine similarity
    ///
    /// Returns patterns ordered by similarity score (descending).
    /// Only patterns with similarity >= threshold are returned.
    pub fn match_by_signature(
        signature: &[f32],
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<PatternMatch>> {
        let all_patterns = Self::list()?;

        let mut matches: Vec<PatternMatch> = all_patterns
            .into_iter()
            .filter_map(|pattern| {
                let similarity = cosine_similarity(signature, &pattern.signature_vector);
                if similarity >= threshold {
                    Some(PatternMatch {
                        pattern,
                        similarity,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity descending
        matches.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        // Limit results
        matches.truncate(limit);

        Ok(matches)
    }

    /// Search patterns by keywords
    pub fn search_by_keywords(query: &str) -> Result<Vec<FixPattern>> {
        let all_patterns = Self::list()?;
        let query_lower = query.to_lowercase();

        let matches: Vec<FixPattern> = all_patterns
            .into_iter()
            .filter(|p| {
                p.keywords.to_lowercase().contains(&query_lower)
                    || p.name.to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(matches)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Calculate cosine similarity between two vectors
///
/// Returns a value in [0, 1] where 1 means identical and 0 means orthogonal.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Identical vectors
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        // Orthogonal vectors
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);

        // Opposite vectors
        let a = vec![1.0, 1.0];
        let b = vec![-1.0, -1.0];
        assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 0.001);

        // Empty vectors
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }
}
