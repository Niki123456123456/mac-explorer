use std::{cmp::Ordering, fs, io, path::Path};

use chrono::{DateTime, Utc};


pub fn get_meta(path: &str) -> io::Result<FileEntry> {
    let p = Path::new(path);
    let meta = fs::metadata(path)?;
    let file_type = meta.file_type();
    let created: DateTime<Utc> = meta.created()?.into();
    let modified: DateTime<Utc> = meta.modified()?.into();
    let accessed: DateTime<Utc> = meta.accessed()?.into();
    let len = meta.len();

    return Ok(FileEntry {
        len,
        file_type,
        created,
        modified,
        accessed,
        path: path.to_string(),
        file_name: p
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string(),
    });
}

pub fn get_entries(path: &str) -> io::Result<Vec<FileEntry>> {
    let mut files = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path().to_str().unwrap_or_default().to_string();
        let file_name = entry.file_name().into_string().unwrap_or_default();
        let meta = entry.metadata()?;
        let file_type = meta.file_type();
        let created: DateTime<Utc> = meta.created()?.into();
        let modified: DateTime<Utc> = meta.modified()?.into();
        let accessed: DateTime<Utc> = meta.accessed()?.into();
        let len = meta.len();

        files.push(FileEntry {
            len,
            file_type,
            created,
            modified,
            accessed,
            path,
            file_name,
        });
    }
    files.sort_by(|a, b| {
        let type_ord = a.file_type.is_file().cmp(&b.file_type.is_file());
        if type_ord == Ordering::Equal {
            return a.file_name.cmp(&b.file_name);
        }
        return type_ord;
    });
    Ok(files)
}

pub fn bytes_to_human_readable(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.0} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.0} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.0} B", bytes)
    }
}

#[derive(Debug)]
pub struct FileEntry {
    pub len: u64,
    pub file_type: fs::FileType,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub accessed: DateTime<Utc>,
    pub path: String,
    pub file_name: String,
}
