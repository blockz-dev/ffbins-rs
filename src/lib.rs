//! FFBins - A rust crate for static FFmpeg binaries
//! ---
//! ##### Currently supports on
//! + **windows:** `ffmpeg`
//! + **linux:** `ffmpeg`
//! + **macos:** `ffmpeg`, `ffprobe`, `ffplay`, `ffserver`
//! 
//! ##### Sources
//! + **macos**   [evermeet.cx/ffmpeg](https://evermeet.cx/ffmpeg/)
//! + **linux**   [github.com/BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds)
//! + **windows** [github.com/BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds)
//!

mod download;
mod extract;
mod sources;
mod utils;



pub use extract::Extractor;
pub use sources::{Binary, Versions};

use std::{path::PathBuf, sync::Arc};
pub use std::process::Command;



#[derive(Debug, Clone)]
pub enum State {
    NotReady,
    Downloading,
    Downloaded,
    Extracting,
    Extracted,
    Ready
}

impl State {
    pub fn to_string(&self) -> String {
        match self {
            State::NotReady => "not ready".to_string(),
            State::Downloading => "downloading".to_string(),
            State::Downloaded => "downloaded".to_string(),
            State::Extracting => "extracting".to_string(),
            State::Extracted => "extracted".to_string(),
            State::Ready => "ready".to_string(),
        }
    }
}














#[derive(Debug, Clone)]
#[allow(unused)]
/// ### The options for the ffbins crate
pub struct Options {
    /// #### Path to store the binaries
    pub dest: PathBuf,
    /// #### Path to store the temporary files for downloading and extracting
    pub temp: PathBuf,
    /// #### The binary to download and use
    pub binary: Binary,
    /// #### The version of the binary
    pub version: Versions
}

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub name: String,
    pub status: State,
    pub current: u64,
    pub total: u64,
    pub percent: f64
}


#[derive(Debug, Clone)]
pub struct FFbins {
    options: Options,
    binary_path: PathBuf,
    commander: Option<Commander>
}

impl FFbins {

    pub fn new(options: Options) -> Self {
        Self {
            options,
            binary_path: PathBuf::new(),
            commander: None
        }
    }

    pub fn init(&mut self) -> Result<()> {

        std::fs::create_dir_all(self.options.dest.clone())?;
        std::fs::create_dir_all(self.options.temp.clone())?;

        self.binary_path = PathBuf::from(self.options.dest.clone()).join(self.options.binary.to_str());

        Ok(())
    }
    
    pub fn commander(&mut self) -> &mut Commander {
        if self.commander.is_none() {
            self.commander = Some(Commander::new(self.binary_path.clone()));
        }
        self.commander.as_mut().unwrap()
    }

    pub fn check(&mut self) -> bool {
        std::fs::metadata(self.binary_path.clone()).is_ok()
    }





    pub fn install<'a, T>(&mut self, callback: &mut T) -> Result<()>
    where
        Self: Clone + Sized,
        T: FnMut(InstallProgress) + Send + Sync + 'a,
    {

        let info = sources::get_url(self.options.binary.clone(), self.options.version.clone()).ok_or(Error::Unknown("failed to get binary download url.".into()))?;

        let mut prog = InstallProgress {
            name: String::new(),
            status: State::Downloading,
            current: 0,
            total: 0,
            percent: 0.0
        };

        download::download_with_progress(&info.url, self.options.temp.to_str().unwrap(), |progress| {

            prog.status = State::Downloading;
            prog.current = progress.current;
            prog.total = progress.total;
            prog.percent = progress.percent;
            callback(prog.clone());

        })?;

        //status = State::Downloaded;
        //callback(InstallProgress { name: name.clone(), status: status.clone(), current, total, percent });

        let extractor = Extractor::load(self.options.temp.join(&info.url).display().to_string(), self.options.temp.display().to_string())?;
        
        extractor.extract(|progress| {

            prog.status = State::Extracting;
            prog.current = progress.current;
            prog.total = progress.total;
            prog.percent = progress.percent;
            callback(prog.clone());

        })?;

        //status = State::Extracted;
        //callback(InstallProgress { name: name.clone(), status: status.clone(), current, total, percent });

        Ok(())

    }




}




#[derive(Debug, Clone)]
pub struct Commander {
    // TODO: Commands
    #[allow(unused)]
    cmd: Arc<Command>
}

impl Commander {
    
    pub fn new(bin: PathBuf) -> Self {
        Self {
            cmd: Arc::new(Command::new(bin))
        }
    }
    
    pub fn install<T>(&mut self, callback: T)
    where
        Self: Sized,
        T: Fn(u64, u64, f64),
    {
        callback(0, 0, 0.0);
    }

}
























// Errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    // std errors
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    // std errors
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    
    // std errors
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    
    // std errors
    #[error(transparent)]
    SevenZipError(#[from] sevenz_rust::Error),

    // lib errors
    #[error("ffmpeg is not ready. current state: {0}")]
    FFmpegNotReady(String),

    // lib errors
    #[error("binary [{0}] not found in PATH")]
    LibBinaryPathError(String),

    // lib errors
    #[error("archive type {0} not supported")]
    UnsupportedArchiveType(String),

    // other errors
    #[error("{0}")]
    CustomLibError(String),
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl serde::Serialize for Error {
fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
S: serde::Serializer,
{
serializer.serialize_str(self.to_string().as_ref())
}
}

pub type Result<T> = std::result::Result<T, Error>;