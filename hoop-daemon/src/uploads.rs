//! Resumable chunked upload API (tus-like protocol)
//!
//! Protocol flow:
//! 1. POST /api/uploads - initiate upload with metadata (filename, size, checksum)
//! 2. PATCH /api/uploads/{upload_id} - upload chunk(s) with Content-Range header
//! 3. HEAD /api/uploads/{upload_id} - check upload progress
//! 4. POST /api/uploads/{upload_id}/complete - finalize and verify checksum
//! 5. On success, file is moved to attachment storage
//!
//! Chunk state is stored in ~/.hoop/uploads/{upload_id}/

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Upload state stored alongside the partial file
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UploadMetadata {
    pub upload_id: String,
    pub filename: String,
    pub total_size: u64,
    pub received_size: u64,
    pub checksum: String,  // hex-encoded SHA-256
    pub attachment_type: String,  // "bead" or "stitch"
    pub resource_id: String,  // bead_id or stitch_id
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Response for upload initiation
#[derive(Debug, serde::Serialize)]
pub struct InitUploadResponse {
    pub upload_id: String,
    pub upload_url: String,
    pub chunk_size: u64,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Response for upload progress query
#[derive(Debug, serde::Serialize)]
pub struct UploadProgressResponse {
    pub upload_id: String,
    pub received_size: u64,
    pub total_size: u64,
    pub offset: u64,
}

/// Upload configuration
#[derive(Debug, Clone)]
pub struct UploadConfig {
    pub chunk_size: u64,
    pub max_file_size: u64,
    pub upload_ttl_hours: i64,
    pub uploads_dir: PathBuf,
}

impl Default for UploadConfig {
    fn default() -> Self {
        let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.push(".hoop");
        home.push("uploads");
        Self {
            chunk_size: 5 * 1024 * 1024,  // 5 MB chunks
            max_file_size: 2 * 1024 * 1024 * 1024,  // 2 GB max
            upload_ttl_hours: 24,  // uploads expire after 24 hours
            uploads_dir: home,
        }
    }
}

/// Upload registry for tracking active uploads
#[derive(Debug, Clone)]
pub struct UploadRegistry {
    config: UploadConfig,
}

impl UploadRegistry {
    pub fn new(config: UploadConfig) -> Result<Self> {
        fs::create_dir_all(&config.uploads_dir)
            .context("failed to create uploads directory")?;
        Ok(Self { config })
    }

    /// Get directory for a specific upload
    fn upload_dir(&self, upload_id: &str) -> Result<PathBuf> {
        let dir = self.config.uploads_dir.join(upload_id);
        // Validate upload_id is a UUID to prevent path traversal
        Uuid::parse_str(upload_id)
            .context("invalid upload ID format")?;
        Ok(dir)
    }

    /// Get metadata file path for an upload
    fn metadata_path(&self, upload_id: &str) -> Result<PathBuf> {
        Ok(self.upload_dir(upload_id)?.join("metadata.json"))
    }

    /// Get partial file path for an upload
    fn partial_path(&self, upload_id: &str) -> Result<PathBuf> {
        Ok(self.upload_dir(upload_id)?.join("partial.bin"))
    }

    /// Load metadata for an upload
    pub fn load_metadata(&self, upload_id: &str) -> Result<UploadMetadata> {
        let meta_path = self.metadata_path(upload_id)?;
        let content = fs::read_to_string(&meta_path)
            .with_context(|| format!("upload not found: {}", upload_id))?;
        let meta: UploadMetadata = serde_json::from_str(&content)
            .context("failed to parse upload metadata")?;
        Ok(meta)
    }

    /// Save metadata for an upload
    fn save_metadata(&self, meta: &UploadMetadata) -> Result<()> {
        let meta_path = self.metadata_path(&meta.upload_id)?;
        let content = serde_json::to_string_pretty(meta)?;
        fs::write(&meta_path, content)
            .context("failed to write metadata")?;
        Ok(())
    }

    /// Initiate a new upload
    pub fn initiate_upload(
        &self,
        filename: String,
        total_size: u64,
        checksum: String,
        attachment_type: String,
        resource_id: String,
    ) -> Result<InitUploadResponse> {
        // Validate inputs
        if total_size > self.config.max_file_size {
            anyhow::bail!("file size {} exceeds maximum {}", total_size, self.config.max_file_size);
        }
        if total_size == 0 {
            anyhow::bail!("file size must be positive");
        }
        if filename.is_empty() || filename.len() > 255 {
            anyhow::bail!("filename must be 1-255 characters");
        }
        // Validate checksum is a hex string of 64 chars (SHA-256 = 32 bytes = 64 hex chars)
        if checksum.len() != 64 || !checksum.chars().all(|c| c.is_ascii_hexdigit()) {
            anyhow::bail!("checksum must be 64-character hex string (SHA-256)");
        }

        let upload_id = Uuid::new_v4().to_string();
        let upload_dir = self.upload_dir(&upload_id)?;
        fs::create_dir_all(&upload_dir)
            .context("failed to create upload directory")?;

        let now = chrono::Utc::now();
        let meta = UploadMetadata {
            upload_id: upload_id.clone(),
            filename,
            total_size,
            received_size: 0,
            checksum,
            attachment_type,
            resource_id,
            created_at: now,
            updated_at: now,
        };

        self.save_metadata(&meta)?;

        // Create empty partial file
        let partial_path = self.partial_path(&upload_id)?;
        File::create(&partial_path)
            .context("failed to create partial file")?;

        let expires_at = now + chrono::Duration::hours(self.config.upload_ttl_hours);
        let upload_url = format!("/api/uploads/{}", upload_id);

        Ok(InitUploadResponse {
            upload_id,
            upload_url,
            chunk_size: self.config.chunk_size,
            expires_at,
        })
    }

    /// Append a chunk to an upload
    pub fn append_chunk(
        &self,
        upload_id: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<UploadProgressResponse> {
        let mut meta = self.load_metadata(upload_id)?;

        // Validate offset
        if offset != meta.received_size {
            anyhow::bail!(
                "offset mismatch: expected {}, got {}",
                meta.received_size,
                offset
            );
        }

        // Check size limit
        let new_size = meta.received_size + data.len() as u64;
        if new_size > meta.total_size {
            anyhow::bail!(
                "chunk would exceed file size: {} + {} > {}",
                meta.received_size,
                data.len(),
                meta.total_size
            );
        }

        // Append to partial file
        let partial_path = self.partial_path(upload_id)?;
        let mut file = OpenOptions::new()
            .write(true)
            .open(&partial_path)
            .context("failed to open partial file")?;

        file.seek(SeekFrom::Start(offset))
            .context("failed to seek in partial file")?;
        file.write_all(data)
            .context("failed to write chunk")?;
        file.sync_all()
            .context("failed to sync chunk to disk")?;

        // Update metadata
        meta.received_size = new_size;
        meta.updated_at = chrono::Utc::now();
        self.save_metadata(&meta)?;

        Ok(UploadProgressResponse {
            upload_id: upload_id.to_string(),
            received_size: meta.received_size,
            total_size: meta.total_size,
            offset: meta.received_size,
        })
    }

    /// Get upload progress
    pub fn get_progress(&self, upload_id: &str) -> Result<UploadProgressResponse> {
        let meta = self.load_metadata(upload_id)?;
        Ok(UploadProgressResponse {
            upload_id: upload_id.to_string(),
            received_size: meta.received_size,
            total_size: meta.total_size,
            offset: meta.received_size,
        })
    }

    /// Complete upload and verify checksum
    pub fn complete_upload(&self, upload_id: &str) -> Result<PathBuf> {
        let meta = self.load_metadata(upload_id)?;
        let partial_path = self.partial_path(upload_id)?;

        // Verify size
        if meta.received_size != meta.total_size {
            anyhow::bail!(
                "incomplete upload: {} of {} bytes received",
                meta.received_size,
                meta.total_size
            );
        }

        // Compute checksum
        let computed_checksum = self.compute_checksum(&partial_path)?;

        if computed_checksum != meta.checksum {
            anyhow::bail!(
                "checksum mismatch: expected {}, computed {}",
                meta.checksum,
                computed_checksum
            );
        }

        // Move to final destination based on attachment type
        let final_path = match meta.attachment_type.as_str() {
            "bead" => {
                let workspace = std::env::current_dir()
                    .context("failed to get current directory")?;
                crate::attachments::bead_attachment_path(
                    &workspace,
                    &meta.resource_id,
                    &meta.filename,
                )?
            }
            "stitch" => {
                crate::attachments::stitch_attachment_path(
                    &meta.resource_id,
                    &meta.filename,
                )?
            }
            _ => anyhow::bail!("invalid attachment type: {}", meta.attachment_type),
        };

        // Atomic rename
        fs::rename(&partial_path, &final_path)
            .with_context(|| format!("failed to move {} to {}", partial_path.display(), final_path.display()))?;

        // Clean up upload directory
        let upload_dir = self.upload_dir(upload_id)?;
        fs::remove_dir_all(&upload_dir)
            .context("failed to clean up upload directory")?;

        Ok(final_path)
    }

    /// Cancel and cleanup an upload
    pub fn cancel_upload(&self, upload_id: &str) -> Result<()> {
        let upload_dir = self.upload_dir(upload_id)?;
        if upload_dir.exists() {
            fs::remove_dir_all(&upload_dir)
                .context("failed to remove upload directory")?;
        }
        Ok(())
    }

    /// Compute SHA-256 checksum of a file
    fn compute_checksum(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path)
            .context("failed to open file for checksum")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file.read(&mut buffer)
                .context("failed to read file for checksum")?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Clean up expired uploads
    pub fn cleanup_expired(&self) -> Result<Vec<String>> {
        let now = chrono::Utc::now();
        let mut cleaned = Vec::new();

        let entries = fs::read_dir(&self.config.uploads_dir)
            .context("failed to read uploads directory")?;

        for entry in entries {
            let entry = entry?;
            let dir_path = entry.path();
            if !dir_path.is_dir() {
                continue;
            }

            let meta_path = dir_path.join("metadata.json");
            if let Ok(content) = fs::read_to_string(&meta_path) {
                if let Ok(meta) = serde_json::from_str::<UploadMetadata>(&content) {
                    let age = now.signed_duration_since(meta.updated_at);
                    if age.num_hours() > self.config.upload_ttl_hours {
                        fs::remove_dir_all(&dir_path)
                            .with_context(|| format!("failed to remove expired upload {}", meta.upload_id))?;
                        cleaned.push(meta.upload_id);
                    }
                }
            }
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config() -> (UploadConfig, TempDir) {
        let tmp = TempDir::new().unwrap();
        let config = UploadConfig {
            chunk_size: 1024,
            max_file_size: 10 * 1024,
            upload_ttl_hours: 24,
            uploads_dir: tmp.path().join("uploads"),
        };
        (config, tmp)
    }

    #[test]
    fn initiate_upload_creates_metadata() {
        let (config, _tmp) = test_config();
        let registry = UploadRegistry::new(config).unwrap();

        let response = registry.initiate_upload(
            "test.txt".to_string(),
            100,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "bead".to_string(),
            "test-bead.1".to_string(),
        ).unwrap();

        assert_eq!(response.chunk_size, 1024);
        assert!(!response.upload_id.is_empty());

        // Verify metadata exists
        let meta = registry.load_metadata(&response.upload_id).unwrap();
        assert_eq!(meta.filename, "test.txt");
        assert_eq!(meta.total_size, 100);
        assert_eq!(meta.received_size, 0);
    }

    #[test]
    fn append_chunk_updates_progress() {
        let (config, _tmp) = test_config();
        let registry = UploadRegistry::new(config).unwrap();

        let init = registry.initiate_upload(
            "test.txt".to_string(),
            100,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "bead".to_string(),
            "test-bead.1".to_string(),
        ).unwrap();

        let data = vec![b'a'; 50];
        let progress = registry.append_chunk(&init.upload_id, 0, &data).unwrap();

        assert_eq!(progress.received_size, 50);
        assert_eq!(progress.offset, 50);

        // Append rest
        let data2 = vec![b'b'; 50];
        let progress2 = registry.append_chunk(&init.upload_id, 50, &data2).unwrap();

        assert_eq!(progress2.received_size, 100);
    }

    #[test]
    fn append_chunk_rejects_wrong_offset() {
        let (config, _tmp) = test_config();
        let registry = UploadRegistry::new(config).unwrap();

        let init = registry.initiate_upload(
            "test.txt".to_string(),
            100,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "bead".to_string(),
            "test-bead.1".to_string(),
        ).unwrap();

        let data = vec![b'a'; 50];
        let result = registry.append_chunk(&init.upload_id, 10, &data);

        assert!(result.is_err());
    }

    #[test]
    fn get_progress_returns_current_state() {
        let (config, _tmp) = test_config();
        let registry = UploadRegistry::new(config).unwrap();

        let init = registry.initiate_upload(
            "test.txt".to_string(),
            100,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            "bead".to_string(),
            "test-bead.1".to_string(),
        ).unwrap();

        let progress = registry.get_progress(&init.upload_id).unwrap();
        assert_eq!(progress.received_size, 0);
        assert_eq!(progress.total_size, 100);
    }
}
