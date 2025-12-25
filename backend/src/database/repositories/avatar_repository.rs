use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::AvatarBlob;

pub struct AvatarRepository;

impl AvatarRepository {
    pub fn upsert_avatar(
        conn: &mut DbConn,
        player_id: i32,
        size: &str,
        image_data: &[u8],
        source_url: &str,
        source_url_hash: &str,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let sql = "
            INSERT INTO avatars (player_id, size, image_data, format, source_url, source_url_hash, width, height, updated_at)
            VALUES (?1, ?2, ?3, 'webp', ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
            ON CONFLICT(player_id, size) DO UPDATE SET
                image_data = excluded.image_data,
                source_url = excluded.source_url,
                source_url_hash = excluded.source_url_hash,
                width = excluded.width,
                height = excluded.height,
                updated_at = CURRENT_TIMESTAMP
        ";

        conn.execute(
            sql,
            params![player_id, size, image_data, source_url, source_url_hash, width, height],
        )
        .context("Failed to upsert avatar")?;

        Ok(())
    }

    pub fn get_avatar(
        conn: &mut DbConn,
        player_id: i32,
        size: &str,
    ) -> Result<Option<AvatarBlob>> {
        let sql = "
            SELECT id, player_id, size, image_data, format, source_url, source_url_hash, width, height, created_at, updated_at
            FROM avatars
            WHERE player_id = ?1 AND size = ?2
        ";

        conn.query_row(sql, params![player_id, size], |row| {
            Ok(AvatarBlob {
                id: row.get(0)?,
                player_id: row.get(1)?,
                size: row.get(2)?,
                image_data: row.get(3)?,
                format: row.get(4)?,
                source_url: row.get(5)?,
                source_url_hash: row.get(6)?,
                width: row.get(7)?,
                height: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .optional()
        .context("Failed to get avatar")
    }

    pub fn get_avatar_hash(
        conn: &mut DbConn,
        player_id: i32,
        size: &str,
    ) -> Result<Option<String>> {
        let sql = "
            SELECT source_url_hash
            FROM avatars
            WHERE player_id = ?1 AND size = ?2
        ";

        conn.query_row(sql, params![player_id, size], |row| row.get(0))
            .optional()
            .context("Failed to get avatar hash")
    }
}
