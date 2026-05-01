use crate::{
    error::AppError,
    outputs::audiobook::{AudiobookLong, AudiobookShort},
    repo::audiobook as audiobook_repo,
    state::AppState,
};
use std::path::PathBuf;

pub async fn list_audiobooks(state: &AppState) -> Result<Vec<AudiobookShort>, AppError> {
    let audiobooks = audiobook_repo::find_all(&state.db).await?;

    Ok(audiobooks
        .into_iter()
        .map(|book| AudiobookShort {
            hash: book.hash,
            title: book.title,
            author: book.author,
            date: book.date,
            genres: serde_json::from_str(&book.genres).unwrap_or_default(),
            duration: book.duration,
            archive_ready: book.archive_ready,
        })
        .collect())
}

pub async fn get_audiobook_by_hash(hash: String, state: &AppState) -> Result<AudiobookLong, AppError> {
    let book = audiobook_repo::find_by_hash(&state.db, &hash)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(AudiobookLong {
        hash: book.hash,
        title: book.title,
        author: book.author,
        date: book.date,
        description: book.description,
        genres: serde_json::from_str(&book.genres).unwrap_or_default(),
        duration: book.duration,
        size: book.size,
        archive_ready: book.archive_ready,
    })
}

pub async fn get_audiobook_archive(hash: String, state: &AppState) -> Result<(String, PathBuf), AppError> {
    let audiobook = audiobook_repo::find_by_hash(&state.db, &hash)
        .await?
        .ok_or(AppError::NotFound)?;

    if !audiobook.archive_ready {
        let mut q = state.archive_queue.lock().unwrap();
        if q.in_progress.as_deref() != Some(&hash) {
            q.pending.retain(|h| h != &hash);
            q.pending.push_front(hash.clone());
            drop(q);
            state.archive_notify.notify_one();
        }
        return Err(AppError::ServiceUnavailable);
    }

    let archive_path = PathBuf::from(&audiobook.path).join("archive.tar.gz");
    let filename = format!(
        "{}-{}-{}.tar.gz",
        slugify(&audiobook.author),
        slugify(&audiobook.title),
        audiobook.date
    );

    Ok((filename, archive_path))
}

fn slugify(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    raw.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
