use serde::Deserialize;
use crate::utils::http_client;


#[derive(Debug, Clone)]
pub enum Binary {
    FFmpeg,
    #[cfg(all(target_os = "macos"))]
    FFprobe,
    #[cfg(all(target_os = "macos"))]
    FFplay,
    #[cfg(all(target_os = "macos"))]
    FFserver
}

impl Binary {
    pub fn to_str(&self) -> &str {
        match self {
            Binary::FFmpeg   => "ffmpeg",
            #[cfg(all(target_os = "macos"))]
            Binary::FFprobe  => "ffprobe",
            #[cfg(all(target_os = "macos"))]
            Binary::FFplay   => "ffplay",
            #[cfg(all(target_os = "macos"))]
            Binary::FFserver => "ffserver",
        }
    }
}



#[derive(Debug, Clone)]
pub enum Versions {
    V8_0,
    V7_1,
}

impl Versions {
    pub fn to_str(&self) -> &str {
        match self {
            #[cfg(not(target_os = "macos"))]
            Versions::V8_0 => "8.0",
            #[cfg(not(target_os = "macos"))]
            Versions::V7_1 => "7.1",
            #[cfg(target_os = "macos")]
            Versions::V8_0 => "8.0.0",
            #[cfg(target_os = "macos")]
            Versions::V7_1 => "7.1.0",
        }
    }
}



#[cfg(target_os = "macos")]
static DOWNLOAD_URL: &'static str = "https://evermeet.cx/ffmpeg/info/{BIN}/{VER}";
#[cfg(not(target_os = "macos"))]
static DOWNLOAD_URL: &'static str = "https://api.github.com/repos/BtbN/FFmpeg-Builds/releases/latest";   





#[cfg(all(not(target_os = "macos"), not(target_os = "windows"), target_arch = "aarch64"))]
static TARGET: &str = "linuxarm64";
#[cfg(all(not(target_os = "macos"), not(target_os = "windows"), target_arch = "x86_64"))]
static TARGET: &str = "linux64";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
static TARGET: &str = "win64";
#[cfg(all(target_os = "windows", target_arch = "aarch64"))]
static TARGET: &str = "winarm64";
#[cfg(target_os = "macos")]
static TARGET: &str = "macos";




#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
static EXT: &str = "tar.xz";
#[cfg(target_os = "windows")]
static EXT: &str = "zip";
#[cfg(target_os = "macos")]
static EXT: &str = "7z";














#[cfg(target_os = "macos")]
#[derive(Deserialize)]
struct Release {
    name: String,
    version: String,
    downloads: ReleaseDownloads,
}

#[cfg(target_os = "macos")]
#[derive(Deserialize)]
struct ReleaseDownloads {
    #[serde(rename = "7z")]
    sevenz: Info
}

#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub url: String,
    pub size: u64,
}

#[cfg(target_os = "macos")]
pub fn get_url_(name: Binary, version: Versions) -> Option<Info> {
    let url = DOWNLOAD_URL.replace("{BIN}", &name.to_string()).replace("{VER}", &version.to_string());
    let json: Release = http_client(url.clone()).unwrap();
    Some(json.downloads.sevenz)
}














#[cfg(not(target_os = "macos"))]
#[derive(Deserialize)]
struct Release {
    assets: Vec<Info>,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    #[serde(rename = "browser_download_url")]
    pub url: String,
    #[allow(unused)]
    pub size: u64,
}

#[cfg(not(target_os = "macos"))]
pub fn get_url(name: Binary, version: Versions) -> Option<Info> {
    let result: Release = http_client(DOWNLOAD_URL).unwrap();

    let starts = format!("{}-n{}", name.to_str(), version.to_str());
    let ends = format!("{}-lgpl-{}.{}", TARGET, version.to_str(), EXT);

    for info in result.assets.iter() {
        println!("{}", info.url);
        if info.url.contains(&starts) && info.url.ends_with(&ends) {
            return Some(info.clone());
        }
    }
    None
}