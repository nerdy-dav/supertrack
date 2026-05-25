use axum::extract::{State, Multipart};
use axum::response::Response;
use axum::body::Body;
use std::sync::Arc;
use std::io::{Write, Read};

use crate::AppState;

pub async fn download(State(state): State<Arc<AppState>>) -> Response<Body> {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    let filename = format!("supertrack-backup-{}.zip", today);

    let temp_dir = std::env::temp_dir();
    let temp_db = temp_dir.join(format!("supertrack-backup-{}.db", today));

    let result = (|| -> anyhow::Result<Vec<u8>> {
        {
            let conn = state.db.lock();
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
            std::fs::copy(&state.db_path, &temp_db)?;
        }

        let db_bytes = std::fs::read(&temp_db)?;
        std::fs::remove_file(&temp_db)?;

        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            zip.start_file("supertrack.db", options)?;
            zip.write_all(&db_bytes)?;
            zip.finish()?;
        }
        Ok(buf.into_inner())
    })();

    match result {
        Ok(bytes) => Response::builder()
            .header("Content-Type", "application/zip")
            .header("Content-Disposition", format!("attachment; filename=\"{}\"", filename))
            .body(Body::from(bytes))
            .unwrap(),
        Err(e) => Response::builder()
            .header("Content-Type", "text/plain")
            .body(Body::from(format!("Backup failed: {}", e)))
            .unwrap(),
    }
}

pub async fn restore(State(state): State<Arc<AppState>>, mut multipart: Multipart) -> Response<Body> {
    let today = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let temp_dir = std::env::temp_dir();

    let mut zip_bytes: Option<Vec<u8>> = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("backup") {
            zip_bytes = field.bytes().await.ok().map(|b| b.to_vec());
        }
    }
    let Some(zip_bytes) = zip_bytes else {
        return error_response("No backup file uploaded. Select a .zip file and try again.");
    };

    let result = (|| -> anyhow::Result<()> {
        let zip_path = temp_dir.join(format!("supertrack-restore-{}.zip", today));
        std::fs::write(&zip_path, &zip_bytes)?;

        let extracted = temp_dir.join(format!("supertrack-restore-{}.db", today));
        {
            let file = std::fs::File::open(&zip_path)?;
            let mut archive = zip::ZipArchive::new(file)?;
            let mut found = false;
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                if entry.name() == "supertrack.db" {
                    let mut contents = Vec::new();
                    entry.read_to_end(&mut contents)?;
                    std::fs::write(&extracted, &contents)?;
                    found = true;
                    break;
                }
            }
            if !found {
                anyhow::bail!("Backup zip does not contain supertrack.db");
            }
        }
        std::fs::remove_file(&zip_path)?;

        let backup_conn = rusqlite::Connection::open(&extracted)?;

        let backup_path = format!("{}.{}.bak", state.db_path, today);
        {
            let conn = state.db.lock();
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }

        if std::path::Path::new(&state.db_path).exists() {
            std::fs::copy(&state.db_path, &backup_path)?;
        }

        {
            let mut live = state.db.lock();
            let backup = rusqlite::backup::Backup::new(&backup_conn, &mut *live)?;
            backup.run_to_completion(100, std::time::Duration::from_millis(250), None)?;
        }

        let _ = std::fs::remove_file(&extracted);
        Ok(())
    })();

    match result {
        Ok(()) => Response::builder()
            .header("Content-Type", "text/html")
            .body(Body::from(
                r#"<!DOCTYPE html>
<html><body>
<p>Restore complete. The previous database was backed up as a .bak file.</p>
<p><a href="/reports">Back to Reports</a></p>
</body></html>"#,
            ))
            .unwrap(),
        Err(e) => error_response(&format!("Restore failed: {}", e)),
    }
}

fn error_response(msg: &str) -> Response<Body> {
    Response::builder()
        .header("Content-Type", "text/plain")
        .body(Body::from(msg.to_string()))
        .unwrap()
}
