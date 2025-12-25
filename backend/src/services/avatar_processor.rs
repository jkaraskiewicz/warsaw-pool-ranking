use anyhow::{Context, Result};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageFormat};
use sha2::{Digest, Sha256};
use std::io::Cursor;

use crate::database::DbConn;
use crate::database::repositories::avatar_repository::AvatarRepository;
use crate::http::RateLimitedClient;

const USER_AGENT: &str = "WarsawPoolRankings/2.0";
const TIMEOUT_SECS: u64 = 30;
const RATE_LIMIT_MS: u64 = 100;

pub struct AvatarProcessor {
    http_client: RateLimitedClient,
}

impl AvatarProcessor {
    pub fn new() -> Result<Self> {
        let http_client = RateLimitedClient::new(USER_AGENT, TIMEOUT_SECS, RATE_LIMIT_MS)?;
        Ok(Self { http_client })
    }

    /// Smart refresh: check if avatar changed before downloading
    pub async fn refresh_avatar_if_changed(
        &mut self,
        conn: &mut DbConn,
        player_id: i32,
        source_url: &str,
    ) -> Result<bool> {
        let new_hash = Self::hash_url(source_url);

        // Check if we already have this exact avatar (any size will have same hash)
        if let Some(existing_hash) = AvatarRepository::get_avatar_hash(conn, player_id, "small")? {
            if existing_hash == new_hash {
                log::debug!("Avatar unchanged for player {}, skipping download", player_id);
                return Ok(false);
            }
        }

        // Avatar changed or doesn't exist, download and process
        log::info!("Downloading new/changed avatar for player {}", player_id);
        self.process_avatar(conn, player_id, source_url).await?;
        Ok(true)
    }

    /// Main processing: download, resize to 3 sizes, store in DB
    async fn process_avatar(
        &mut self,
        conn: &mut DbConn,
        player_id: i32,
        source_url: &str,
    ) -> Result<()> {
        // Download image
        let image_data = self.download_avatar(source_url).await?;

        // Decode image
        let img = image::load_from_memory(&image_data)
            .context("Failed to decode image")?;

        let source_url_hash = Self::hash_url(source_url);

        // Process and store each size
        let sizes = [
            ("small", 40u32),
            ("medium", 80u32),
            ("large", 96u32),
        ];

        for (size_name, target_size) in sizes.iter() {
            match self.resize_and_encode(&img, *target_size) {
                Ok((webp_data, width, height)) => {
                    AvatarRepository::upsert_avatar(
                        conn,
                        player_id,
                        size_name,
                        &webp_data,
                        source_url,
                        &source_url_hash,
                        width,
                        height,
                    )?;
                    log::debug!(
                        "Stored {} avatar for player {}: {}x{}",
                        size_name,
                        player_id,
                        width,
                        height
                    );
                }
                Err(e) => {
                    log::warn!(
                        "Failed to process {} avatar for player {}: {:?}",
                        size_name,
                        player_id,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    /// Download avatar from URL using rate-limited client
    async fn download_avatar(&mut self, url: &str) -> Result<Vec<u8>> {
        let response = self
            .http_client
            .get(url)
            .await
            .context("Failed to fetch avatar")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error {}: {}", response.status(), url);
        }

        let bytes = response
            .bytes()
            .await
            .context("Failed to read avatar bytes")?;

        Ok(bytes.to_vec())
    }

    /// Resize and encode to WebP
    /// Returns (webp_data, width, height)
    fn resize_and_encode(&self, img: &DynamicImage, target_size: u32) -> Result<(Vec<u8>, i32, i32)> {
        // Calculate dimensions maintaining aspect ratio with center crop
        let (orig_width, orig_height) = img.dimensions();

        // Determine crop dimensions (square crop from center)
        let crop_size = orig_width.min(orig_height);
        let x_offset = (orig_width - crop_size) / 2;
        let y_offset = (orig_height - crop_size) / 2;

        // Crop to square
        let cropped = img.crop_imm(x_offset, y_offset, crop_size, crop_size);

        // Resize to target size
        let resized = cropped.resize_exact(target_size, target_size, FilterType::Lanczos3);

        // Encode to WebP
        let mut webp_buffer = Cursor::new(Vec::new());
        resized
            .write_to(&mut webp_buffer, ImageFormat::WebP)
            .context("Failed to encode WebP")?;

        let webp_data = webp_buffer.into_inner();
        Ok((webp_data, target_size as i32, target_size as i32))
    }

    /// Generate SHA256 hash of URL for change detection
    fn hash_url(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
