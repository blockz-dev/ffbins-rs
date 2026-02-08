use std::fs::File;
use std::io::{Write, Read};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_LENGTH;
use std::path::{Path, PathBuf};

fn get_filename_from_path(path: &str) -> Option<&str> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
}

fn filename_exists_in_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .is_some()
}

pub struct DownloadStatus {
    pub current: u64,
    pub total: u64,
    pub percent: f64,
}

pub fn download_with_progress<'a, F>(
    url: &str,
    output: &str,
    mut progress_callback: F,
) -> crate::Result<()>
where
    F: FnMut(DownloadStatus) + Send + Sync + 'a,
{
    let client = Client::new();
    let mut response = client.get(url).send()?;

    let total_size = response
        .headers()
        .get(CONTENT_LENGTH)
        .and_then(|ct_len| ct_len.to_str().ok())
        .and_then(|ct_len| ct_len.parse().ok())
        .unwrap_or(0);

    let dest = if filename_exists_in_path(output) {
        PathBuf::from(output)
    } else {
        let filename = get_filename_from_path(output).unwrap();
        Path::new(output).join(filename)
    };

    std::fs::create_dir_all(&dest)?;

    let mut file = File::open(dest)?;

    let mut buffer = vec![0; 1024]; // 1KB buffer
    let mut downloaded = 0u64;

    while let Ok(n) = response.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        let percent = (downloaded as f64 / total_size as f64) * 100.0;
        progress_callback(DownloadStatus {
            current: downloaded,
            total: total_size,
            percent,
        });
    }

    Ok(())
}