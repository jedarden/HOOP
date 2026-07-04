//! Content blocks for multimodal input
//!
//! This module provides the content blocks service for storing and retrieving
//! multimodal content (text, image, audio, video, file) associated with stitches.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tracing::info;
/// Content block type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockType {
    Text,
    Image,
    Audio,
    Video,
    File,
}

impl From<String> for ContentBlockType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "text" => ContentBlockType::Text,
            "image" => ContentBlockType::Image,
            "audio" => ContentBlockType::Audio,
            "video" => ContentBlockType::Video,
            "file" => ContentBlockType::File,
            _ => ContentBlockType::File,
        }
    }
}

impl From<ContentBlockType> for String {
    fn from(t: ContentBlockType) -> Self {
        match t {
            ContentBlockType::Text => "text".to_string(),
            ContentBlockType::Image => "image".to_string(),
            ContentBlockType::Audio => "audio".to_string(),
            ContentBlockType::Video => "video".to_string(),
            ContentBlockType::File => "file".to_string(),
        }
    }
}

/// Content block for multimodal input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ContentBlock {
    pub id: String,
    pub stitch_id: String,
    pub block_type: ContentBlockType,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub block_order: i64,
    pub created_at: String,
}

/// Create content block request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ContentBlockCreate {
    pub block_type: ContentBlockType,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub block_order: Option<i64>,
}

/// Update content block request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ContentBlockUpdate {
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub block_order: Option<i64>,
}

/// Get content blocks for a stitch
pub fn get_content_blocks(conn: &mut Connection, stitch_id: &str) -> Result<Vec<ContentBlock>> {
    let mut stmt = conn.prepare(
        "SELECT id, stitch_id, block_type, content, metadata, block_order, created_at
         FROM content_blocks
         WHERE stitch_id = ?1
         ORDER BY block_order ASC, created_at ASC",
    )?;

    let blocks = stmt
        .query_map(params![stitch_id], |row| {
            Ok(ContentBlock {
                id: row.get(0)?,
                stitch_id: row.get(1)?,
                block_type: ContentBlockType::from(row.get::<_, String>(2)?),
                content: row.get(3)?,
                metadata: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| serde_json::from_str(&s).ok()),
                block_order: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;

    Ok(blocks)
}

/// Create a content block
pub fn create_content_block(conn: &mut Connection, block: &ContentBlock) -> Result<()> {
    let block_type_str: String = block.block_type.clone().into();
    let metadata_json = block
        .metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());

    conn.execute(
        "INSERT INTO content_blocks (id, stitch_id, block_type, content, metadata, block_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            &block.id,
            &block.stitch_id,
            block_type_str,
            &block.content,
            metadata_json,
            block.block_order,
            &block.created_at,
        ],
    )?;

    info!("Created content block {} for stitch {}", block.id, block.stitch_id);
    Ok(())
}

/// Update a content block
pub fn update_content_block(
    conn: &mut Connection,
    block_id: &str,
    update: ContentBlockUpdate,
) -> Result<ContentBlock> {
    let current = get_content_block(conn, block_id)?;

    let metadata_json = if let Some(metadata) = update.metadata {
        Some(serde_json::to_string(&metadata)?)
    } else {
        current.metadata.and_then(|m| serde_json::to_string(&m).ok())
    };

    conn.execute(
        "UPDATE content_blocks
         SET content = COALESCE(?1, content),
             metadata = COALESCE(?2, metadata),
             block_order = COALESCE(?3, block_order)
         WHERE id = ?4",
        params![
            update.content.as_ref().or(current.content.as_ref()),
            metadata_json.as_ref(),
            update.block_order.or(Some(current.block_order)),
            block_id,
        ],
    )?;

    get_content_block(conn, block_id)
}

/// Delete a content block
pub fn delete_content_block(conn: &mut Connection, block_id: &str) -> Result<()> {
    conn.execute("DELETE FROM content_blocks WHERE id = ?1", params![block_id])?;
    info!("Deleted content block {}", block_id);
    Ok(())
}

/// Reorder content blocks for a stitch
pub fn reorder_content_blocks(conn: &mut Connection, stitch_id: &str, ordering: &[String]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    for (index, block_id) in ordering.iter().enumerate() {
        tx.execute(
            "UPDATE content_blocks SET block_order = ?1 WHERE id = ?2 AND stitch_id = ?3",
            params![index as i64, block_id, stitch_id],
        )?;
    }

    tx.commit()?;
    info!("Reordered {} content blocks for stitch {}", ordering.len(), stitch_id);
    Ok(())
}

/// Get a single content block by ID
fn get_content_block(conn: &mut Connection, block_id: &str) -> Result<ContentBlock> {
    let mut stmt = conn.prepare(
        "SELECT id, stitch_id, block_type, content, metadata, block_order, created_at
         FROM content_blocks
         WHERE id = ?1",
    )?;

    let block = stmt.query_row(params![block_id], |row| {
        Ok(ContentBlock {
            id: row.get(0)?,
            stitch_id: row.get(1)?,
            block_type: ContentBlockType::from(row.get::<_, String>(2)?),
            content: row.get(3)?,
            metadata: row
                .get::<_, Option<String>>(4)?
                .and_then(|s| serde_json::from_str(&s).ok()),
            block_order: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

    #[test]
    fn test_content_block_type_conversion() {
        assert_eq!(ContentBlockType::from("text".to_string()), ContentBlockType::Text);
        assert_eq!(ContentBlockType::from("image".to_string()), ContentBlockType::Image);
        assert_eq!(ContentBlockType::from("audio".to_string()), ContentBlockType::Audio);
        assert_eq!(ContentBlockType::from("video".to_string()), ContentBlockType::Video);
        assert_eq!(ContentBlockType::from("file".to_string()), ContentBlockType::File);

        assert_eq!(String::from(ContentBlockType::Text), "text");
        assert_eq!(String::from(ContentBlockType::Image), "image");
        assert_eq!(String::from(ContentBlockType::Audio), "audio");
        assert_eq!(String::from(ContentBlockType::Video), "video");
        assert_eq!(String::from(ContentBlockType::File), "file");
    }
}
