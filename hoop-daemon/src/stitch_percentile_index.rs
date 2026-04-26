//! Historical Stitch percentile indexer for What-Will-This-Take preview
//!
//! Maintains a rolling index over closed Stitches:
//! `(title-tokens, body-length, labels, attachments) → (cost_p50, cost_p90, duration_p50, duration_p90)`
//! by similarity bucket. Updates on Stitch close. Query path feeds the preview card.
//!
//! Acceptance (§6 Phase 4 marquee #8 bullets 2–3):
//! - Bucket size and similarity threshold tuned
//! - Index rebuilds on schema change
//! - Preview query <50ms

use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::similarity::tokenize;

/// Bucket ID for grouping similar Stitches
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BucketId {
    /// Hash of title token set (for similarity grouping)
    pub title_tokens_hash: String,
    /// Body length bucket
    pub body_length_bucket: BodyLengthBucket,
    /// Hash of sorted labels (for label similarity)
    pub labels_hash: String,
    /// Attachments count bucket
    pub attachments_bucket: AttachmentsBucket,
}

impl BucketId {
    /// Compute a bucket ID from Stitch features
    pub fn from_features(
        title: &str,
        body_length: usize,
        labels: &[String],
        attachments_count: usize,
    ) -> Self {
        // Hash title tokens (first 5 unique tokens for bucketing)
        let title_tokens: std::collections::HashSet<_> =
            tokenize(title).into_iter().take(5).collect();
        let title_tokens_hash = {
            let mut tokens: Vec<_> = title_tokens.iter().cloned().collect();
            tokens.sort();
            let mut hasher = Sha256::new();
            hasher.update(tokens.join(","));
            format!("{:x}", hasher.finalize())
        };

        // Body length bucket
        let body_length_bucket = BodyLengthBucket::from_length(body_length);

        // Labels hash (sorted unique labels)
        let labels_hash = {
            let mut sorted_labels: Vec<_> = labels
                .iter()
                .map(|l| l.to_lowercase())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            sorted_labels.sort();
            let mut hasher = Sha256::new();
            hasher.update(sorted_labels.join(","));
            format!("{:x}", hasher.finalize())
        };

        // Attachments bucket
        let attachments_bucket = AttachmentsBucket::from_count(attachments_count);

        BucketId {
            title_tokens_hash,
            body_length_bucket,
            labels_hash,
            attachments_bucket,
        }
    }

    /// Convert to string key for database storage
    pub fn to_key(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.title_tokens_hash,
            self.body_length_bucket.as_str(),
            self.labels_hash,
            self.attachments_bucket.as_str()
        )
    }
}

/// Body length bucket for grouping similar Stitches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyLengthBucket {
    Empty,
    Short,    // 1-100 chars
    Medium,   // 101-500 chars
    Long,     // 501-2000 chars
    VeryLong, // 2000+ chars
}

impl BodyLengthBucket {
    pub fn from_length(len: usize) -> Self {
        match len {
            0 => BodyLengthBucket::Empty,
            1..=100 => BodyLengthBucket::Short,
            101..=500 => BodyLengthBucket::Medium,
            501..=2000 => BodyLengthBucket::Long,
            _ => BodyLengthBucket::VeryLong,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            BodyLengthBucket::Empty => "empty",
            BodyLengthBucket::Short => "short",
            BodyLengthBucket::Medium => "medium",
            BodyLengthBucket::Long => "long",
            BodyLengthBucket::VeryLong => "verylong",
        }
    }
}

/// Attachments count bucket
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttachmentsBucket {
    None,
    One,
    Multiple,
}

impl AttachmentsBucket {
    pub fn from_count(count: usize) -> Self {
        match count {
            0 => AttachmentsBucket::None,
            1 => AttachmentsBucket::One,
            _ => AttachmentsBucket::Multiple,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AttachmentsBucket::None => "none",
            AttachmentsBucket::One => "one",
            AttachmentsBucket::Multiple => "multiple",
        }
    }
}

/// Percentile estimates for a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketPercentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// Number of Stitches in this bucket
    pub count: usize,
}

/// Cost and duration percentiles for a Stitch bucket
#[derive(Debug, Clone)]
pub struct StitchPercentiles {
    /// Cost percentiles in USD
    pub cost: BucketPercentiles,
    /// Duration percentiles in seconds
    pub duration: BucketPercentiles,
    /// Number of similar Stitches
    pub sample_count: usize,
}

/// A closed Stitch's features for indexing
#[derive(Debug, Clone)]
pub struct StitchFeatures {
    pub stitch_id: String,
    pub title: String,
    pub body_length: usize,
    pub labels: Vec<String>,
    pub attachments_count: usize,
    pub cost_usd: f64,
    pub duration_seconds: i64,
}

/// Query result from the percentile index
#[derive(Debug, Clone)]
pub struct PercentileQuery {
    /// Cost percentiles
    pub cost: BucketPercentiles,
    /// Duration percentiles
    pub duration: BucketPercentiles,
    /// Number of Stitches in the bucket
    pub sample_count: usize,
}

/// Initialize the percentile index table
pub fn init_index(conn: &mut Connection) -> Result<()> {
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS stitch_percentile_index (
            bucket_key TEXT PRIMARY KEY NOT NULL,
            title_tokens_hash TEXT NOT NULL,
            body_length_bucket TEXT NOT NULL,
            labels_hash TEXT NOT NULL,
            attachments_bucket TEXT NOT NULL,
            cost_p50 REAL NOT NULL DEFAULT 0.0,
            cost_p90 REAL NOT NULL DEFAULT 0.0,
            duration_p50 REAL NOT NULL DEFAULT 0.0,
            duration_p90 REAL NOT NULL DEFAULT 0.0,
            sample_count INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )?;

    // Index for lookup by bucket features
    conn.execute(
        r#"
        CREATE INDEX IF NOT EXISTS idx_stitch_percentile_lookup
        ON stitch_percentile_index(title_tokens_hash, body_length_bucket, labels_hash, attachments_bucket)
        "#,
        [],
    )?;

    Ok(())
}

/// Update the percentile index with a new closed Stitch
///
/// This should be called when a Stitch is closed to update the rolling
/// percentiles for its similarity bucket.
pub fn update_index(conn: &Connection, stitch: &StitchFeatures) -> Result<()> {
    let bucket_id = BucketId::from_features(
        &stitch.title,
        stitch.body_length,
        &stitch.labels,
        stitch.attachments_count,
    );
    let bucket_key = bucket_id.to_key();

    // First, try to read existing data for this bucket
    let (cost_p50, cost_p90, duration_p50, duration_p90, sample_count) = conn
        .query_row(
            r#"
            SELECT cost_p50, cost_p90, duration_p50, duration_p90, sample_count
            FROM stitch_percentile_index WHERE bucket_key = ?
            "#,
            params![bucket_key],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get::<_, i64>(4)? as usize,
                ))
            },
        )
        .unwrap_or((0.0, 0.0, 0.0, 0.0, 0));

    // Increment sample count and recompute percentiles
    let new_count = sample_count + 1;

    // Online percentile estimation: we maintain the samples and recompute
    // For efficiency, we use a simplified approach: update running estimates
    // In production, you'd want a more sophisticated algorithm (e.g., t-digest)
    let new_cost_p50 = update_percentile_estimate(cost_p50, stitch.cost_usd, new_count);
    let new_cost_p90 = update_percentile_estimate(cost_p90, stitch.cost_usd, new_count);
    let new_duration_p50 =
        update_percentile_estimate(duration_p50, stitch.duration_seconds as f64, new_count);
    let new_duration_p90 =
        update_percentile_estimate(duration_p90, stitch.duration_seconds as f64, new_count);

    let now = Utc::now().to_rfc3339();

    // Insert or replace
    conn.execute(
        r#"
        INSERT INTO stitch_percentile_index
        (bucket_key, title_tokens_hash, body_length_bucket, labels_hash, attachments_bucket,
         cost_p50, cost_p90, duration_p50, duration_p90, sample_count, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (bucket_key) DO UPDATE SET
            cost_p50 = excluded.cost_p50,
            cost_p90 = excluded.cost_p90,
            duration_p50 = excluded.duration_p50,
            duration_p90 = excluded.duration_p90,
            sample_count = excluded.sample_count,
            updated_at = excluded.updated_at
        "#,
        params![
            bucket_key,
            bucket_id.title_tokens_hash,
            bucket_id.body_length_bucket.as_str(),
            bucket_id.labels_hash,
            bucket_id.attachments_bucket.as_str(),
            new_cost_p50,
            new_cost_p90,
            new_duration_p50,
            new_duration_p90,
            new_count as i64,
            now,
        ],
    )?;

    Ok(())
}

/// Update a percentile estimate with a new sample
///
/// Uses a simple moving average approach. For production, consider
/// using a more sophisticated algorithm like t-digest or KLL sketch.
fn update_percentile_estimate(current: f64, new_sample: f64, count: usize) -> f64 {
    if count == 1 {
        return new_sample;
    }
    // Exponential moving average with alpha = 2 / (count + 1)
    // This gives more weight to recent samples
    let alpha = 2.0 / (count as f64 + 1.0);
    current * (1.0 - alpha) + new_sample * alpha
}

/// Query the percentile index for similar Stitches
///
/// Returns the cost and duration percentiles for Stitches similar to
/// the given draft. Returns None if no similar Stitches found.
pub fn query_percentiles(
    conn: &Connection,
    draft_title: &str,
    draft_body_length: usize,
    draft_labels: &[String],
    draft_attachments_count: usize,
) -> Result<Option<PercentileQuery>> {
    let bucket_id = BucketId::from_features(
        draft_title,
        draft_body_length,
        draft_labels,
        draft_attachments_count,
    );

    // Query exact bucket match first
    let result = conn
        .query_row(
            r#"
            SELECT cost_p50, cost_p90, duration_p50, duration_p90, sample_count
            FROM stitch_percentile_index
            WHERE title_tokens_hash = ?1
              AND body_length_bucket = ?2
              AND labels_hash = ?3
              AND attachments_bucket = ?4
            "#,
            params![
                bucket_id.title_tokens_hash,
                bucket_id.body_length_bucket.as_str(),
                bucket_id.labels_hash,
                bucket_id.attachments_bucket.as_str(),
            ],
            |row| {
                Ok(PercentileQuery {
                    cost: BucketPercentiles {
                        p50: row.get(0)?,
                        p90: row.get(1)?,
                        count: row.get::<_, i64>(4)? as usize,
                    },
                    duration: BucketPercentiles {
                        p50: row.get(2)?,
                        p90: row.get(3)?,
                        count: row.get::<_, i64>(4)? as usize,
                    },
                    sample_count: row.get::<_, i64>(4)? as usize,
                })
            },
        );

    match result {
        Ok(q) => Ok(Some(q)),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Try fuzzy match: same title hash and body length, ignore labels/attachments
            let fuzzy_result = conn.query_row(
                r#"
                SELECT cost_p50, cost_p90, duration_p50, duration_p90, sample_count
                FROM stitch_percentile_index
                WHERE title_tokens_hash = ?1
                  AND body_length_bucket = ?2
                LIMIT 1
                "#,
                params![
                    bucket_id.title_tokens_hash,
                    bucket_id.body_length_bucket.as_str(),
                ],
                |row| {
                    Ok(PercentileQuery {
                        cost: BucketPercentiles {
                            p50: row.get(0)?,
                            p90: row.get(1)?,
                            count: row.get::<_, i64>(4)? as usize,
                        },
                        duration: BucketPercentiles {
                            p50: row.get(2)?,
                            p90: row.get(3)?,
                            count: row.get::<_, i64>(4)? as usize,
                        },
                        sample_count: row.get::<_, i64>(4)? as usize,
                    })
                },
            );

            match fuzzy_result {
                Ok(q) => Ok(Some(q)),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                Err(e) => Err(e.into()),
            }
        }
        Err(e) => Err(e.into()),
    }
}

/// Rebuild the entire percentile index from closed Stitches
///
/// This is called on schema changes or when the index needs to be
/// completely rebuilt.
pub fn rebuild_index(conn: &mut Connection) -> Result<()> {
    // Clear existing index
    conn.execute("DELETE FROM stitch_percentile_index", [])?;

    // Collect all Stitches with their features and metrics
    let mut stitches_by_bucket: HashMap<BucketId, Vec<StitchFeatures>> = HashMap::new();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            s.id,
            s.title,
            (SELECT sm.content FROM stitch_messages sm
             WHERE sm.stitch_id = s.id AND sm.role = 'user'
             ORDER BY sm.ts ASC LIMIT 1) AS body,
            (SELECT COALESCE(SUM(sm.tokens), 0) FROM stitch_messages sm
             WHERE sm.stitch_id = s.id) AS total_tokens,
            s.attachments_path
        FROM stitches s
        WHERE s.last_activity_at < datetime('now', '-1 hour')
        ORDER BY s.last_activity_at DESC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let body: Option<String> = row.get(2).unwrap_or(None);
        let total_tokens: i64 = row.get(3).unwrap_or(0);
        let attachments_path: Option<String> = row.get(4).unwrap_or(None);

        // Derive cost and duration from the Stitch
        let cost_usd = (total_tokens as f64) * 30.0 / 1_000_000.0;
        let duration_seconds = 0; // Would be computed from timestamps

        // Extract labels from audit log
        let labels = Vec::new(); // TODO: load from actions table

        Ok(StitchFeatures {
            stitch_id: id,
            title,
            body_length: body.as_ref().map(|b| b.len()).unwrap_or(0),
            labels,
            attachments_count: if attachments_path.is_some() { 1 } else { 0 },
            cost_usd,
            duration_seconds,
        })
    })?;

    for row in rows {
        let stitch = row?;
        let bucket_id = BucketId::from_features(
            &stitch.title,
            stitch.body_length,
            &stitch.labels,
            stitch.attachments_count,
        );
        stitches_by_bucket.entry(bucket_id).or_default().push(stitch);
    }

    // Compute percentiles for each bucket
    for (bucket_id, mut stitches) in stitches_by_bucket {
        if stitches.is_empty() {
            continue;
        }

        // Sort by cost and duration
        stitches.sort_by(|a, b| a.cost_usd.partial_cmp(&b.cost_usd).unwrap());

        let cost_p50 = percentile_at(&stitches, 0.5, |s| s.cost_usd);
        let cost_p90 = percentile_at(&stitches, 0.9, |s| s.cost_usd);

        stitches.sort_by_key(|s| s.duration_seconds);
        let duration_p50 = percentile_at(&stitches, 0.5, |s| s.duration_seconds as f64);
        let duration_p90 = percentile_at(&stitches, 0.9, |s| s.duration_seconds as f64);

        let bucket_key = bucket_id.to_key();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO stitch_percentile_index
            (bucket_key, title_tokens_hash, body_length_bucket, labels_hash, attachments_bucket,
             cost_p50, cost_p90, duration_p50, duration_p90, sample_count, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                bucket_key,
                bucket_id.title_tokens_hash,
                bucket_id.body_length_bucket.as_str(),
                bucket_id.labels_hash,
                bucket_id.attachments_bucket.as_str(),
                cost_p50,
                cost_p90,
                duration_p50,
                duration_p90,
                stitches.len() as i64,
                now,
            ],
        )?;
    }

    Ok(())
}

/// Compute percentile at a given quantile from a sorted slice
fn percentile_at<T, F>(data: &[T], quantile: f64, f: F) -> f64
where
    F: Fn(&T) -> f64,
{
    if data.is_empty() {
        return 0.0;
    }
    let idx = (data.len() as f64 * quantile).floor() as usize;
    let idx = idx.min(data.len() - 1);
    f(&data[idx])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_length_bucket() {
        assert_eq!(BodyLengthBucket::from_length(0), BodyLengthBucket::Empty);
        assert_eq!(BodyLengthBucket::from_length(50), BodyLengthBucket::Short);
        assert_eq!(BodyLengthBucket::from_length(250), BodyLengthBucket::Medium);
        assert_eq!(BodyLengthBucket::from_length(1000), BodyLengthBucket::Long);
        assert_eq!(BodyLengthBucket::from_length(5000), BodyLengthBucket::VeryLong);
    }

    #[test]
    fn test_attachments_bucket() {
        assert_eq!(AttachmentsBucket::from_count(0), AttachmentsBucket::None);
        assert_eq!(AttachmentsBucket::from_count(1), AttachmentsBucket::One);
        assert_eq!(AttachmentsBucket::from_count(5), AttachmentsBucket::Multiple);
    }

    #[test]
    fn test_bucket_id_to_key() {
        let bucket = BucketId {
            title_tokens_hash: "abc123".to_string(),
            body_length_bucket: BodyLengthBucket::Medium,
            labels_hash: "def456".to_string(),
            attachments_bucket: AttachmentsBucket::One,
        };
        let key = bucket.to_key();
        assert_eq!(key, "abc123|medium|def456|one");
    }

    #[test]
    fn test_bucket_id_from_features() {
        let bucket = BucketId::from_features(
            "Fix the crash in the authentication module",
            250,
            &["bug".to_string(), "urgent".to_string()],
            1,
        );
        assert!(!bucket.title_tokens_hash.is_empty());
        assert_eq!(bucket.body_length_bucket, BodyLengthBucket::Medium);
        assert!(!bucket.labels_hash.is_empty());
        assert_eq!(bucket.attachments_bucket, AttachmentsBucket::One);
    }

    #[test]
    fn test_update_percentile_estimate() {
        // First sample
        let p50 = update_percentile_estimate(0.0, 10.0, 1);
        assert_eq!(p50, 10.0);

        // Second sample (average)
        let p50 = update_percentile_estimate(10.0, 20.0, 2);
        assert!((p50 - 15.0).abs() < 0.01);

        // Third sample (weighted average)
        let p50 = update_percentile_estimate(15.0, 30.0, 3);
        // EMA with alpha = 2/(3+1) = 0.5
        // new = 15 * 0.5 + 30 * 0.5 = 22.5
        assert!((p50 - 20.0).abs() < 1.0); // Allow some tolerance
    }

    #[test]
    fn test_percentile_at() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        let p50 = percentile_at(&data, 0.5, |&x| x);
        assert_eq!(p50, 6.0); // index 5 (0.5 * 10 = 5)

        let p90 = percentile_at(&data, 0.9, |&x| x);
        assert_eq!(p90, 10.0); // index 9 (0.9 * 10 = 9)
    }

    #[test]
    fn test_percentile_at_empty() {
        let data: Vec<f64> = vec![];
        let p50 = percentile_at(&data, 0.5, |&x| x);
        assert_eq!(p50, 0.0);
    }
}
