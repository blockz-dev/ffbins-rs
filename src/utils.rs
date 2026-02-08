use std::{env, fs};
use std::path::PathBuf;
use std::time::Duration;
use std::str::FromStr;

use regex::Regex;



static USER_AGENT: &str = "FFBins/1.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.142.86 Safari/537.36";


pub fn http_client<T>(url: &str) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    let json : T = reqwest::blocking::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?
        .get(url)
        .send()?
        .json()?;
    Ok(json)
}








#[allow(unused)]
pub fn process_regex(str: String) -> String {
    let re = Regex::new(r"(\d{2}):(\d{2}):(\d{2})\.(\d{2})").unwrap();
    if let Some(caps) = re.captures(&str) {
        let hours = caps.get(1).unwrap().as_str();
        let minutes = caps.get(2).unwrap().as_str();
        let seconds = caps.get(3).unwrap().as_str();
        let milliseconds = caps.get(4).unwrap().as_str();
        format!("{}:{}:{}.{}", hours, minutes, seconds, milliseconds)
    } else {
        println!("No match found!");
        "00:00:00.00".to_string()
    }
}








#[allow(unused)]
pub fn parse_time(time_str: &str) -> Duration {
    let parts: Vec<&str> = time_str.split(':').collect();

    let hours = u64::from_str(parts[0]).unwrap();
    let minutes = u64::from_str(parts[1]).unwrap();

    let sec_millis: Vec<&str> = parts[2].split('.').collect();
    let seconds = u64::from_str(sec_millis[0]).unwrap();
    let milliseconds = u64::from_str(sec_millis[1]).unwrap();

    let duration = Duration::from_secs(hours * 3600 + minutes * 60 + seconds) + Duration::from_millis(milliseconds);

    duration
}








#[allow(unused)]
pub type FoundPaths = Vec<FoundPath>;

#[allow(unused)]
#[derive(Debug)]
pub struct FoundPath {
    pub path: PathBuf,
    pub binary: String,
    pub full_path: PathBuf,
}

#[allow(unused)]
pub fn find_in_path(binary: &str) -> Option<FoundPaths> {
    let mut found: FoundPaths = Vec::new();
    let envpath = env::var_os("PATH").unwrap();
    let paths = env::split_paths(&envpath);
    for path in paths {
        let name = if cfg!(target_os = "windows") { if binary.ends_with(".exe") { binary } else { &format!("{}.exe", binary) } } else { binary };
        if path.join(name).exists() && fs::metadata(path.join(name)).unwrap().is_file() {
            if !found.iter().any(|p| p.path == path) {
                found.push(FoundPath { path: path.clone(), binary: name.to_string(), full_path: path.join(name) });
            }
        }
    }
    if !found.is_empty() {
        return Some(found)
    }
    None
}
