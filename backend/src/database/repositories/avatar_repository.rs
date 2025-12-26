use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use crate::database::connection::DbConn;
use crate::database::models::AvatarBlob;

/// Parameters for upserting an avatar
pub struct UpsertAvatarParams<'a> {
    pub player_cuescore_id: i64,
    pub size: &'a str,
    pub image_data: &'a [u8],
    pub source_url: &'a str,
    pub source_url_hash: &'a str,
    pub width: i32,
    pub height: i32,
}

pub struct AvatarRepository;

impl AvatarRepository {
    pub fn upsert_avatar(conn: &mut DbConn, params_data: UpsertAvatarParams) -> Result<()> {
        let sql = "
            INSERT INTO avatars (player_cuescore_id, size, image_data, format, source_url, source_url_hash, width, height, updated_at)
            VALUES (?1, ?2, ?3, 'webp', ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
            ON CONFLICT(player_cuescore_id, size) DO UPDATE SET
                image_data = excluded.image_data,
                source_url = excluded.source_url,
                source_url_hash = excluded.source_url_hash,
                width = excluded.width,
                height = excluded.height,
                updated_at = CURRENT_TIMESTAMP
        ";

        conn.execute(
            sql,
            params![
                params_data.player_cuescore_id,
                params_data.size,
                params_data.image_data,
                params_data.source_url,
                params_data.source_url_hash,
                params_data.width,
                params_data.height
            ],
        )
        .context("Failed to upsert avatar")?;

        Ok(())
    }

    pub fn get_avatar(
        conn: &mut DbConn,
        player_cuescore_id: i64,
        size: &str,
    ) -> Result<Option<AvatarBlob>> {
        let sql = "
            SELECT id, player_cuescore_id, size, image_data, format, source_url, source_url_hash, width, height, created_at, updated_at
            FROM avatars
            WHERE player_cuescore_id = ?1 AND size = ?2
        ";

        conn.query_row(sql, params![player_cuescore_id, size], |row| {
            Ok(AvatarBlob {
                id: row.get(0)?,
                player_cuescore_id: row.get(1)?,
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
        player_cuescore_id: i64,
        size: &str,
    ) -> Result<Option<String>> {
        let sql = "
            SELECT source_url_hash
            FROM avatars
            WHERE player_cuescore_id = ?1 AND size = ?2
        ";

        conn.query_row(sql, params![player_cuescore_id, size], |row| row.get(0))
            .optional()
            .context("Failed to get avatar hash")
    }
}
