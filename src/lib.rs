//! FFBins - A rust crate for static FFmpeg binaries
//! ---
//! ##### Currently supports on
//! + **windows:** `ffmpeg`, `ffprobe`, `ffplay`
//! + **linux:** `ffmpeg`, `ffprobe`, `ffplay`
//! + **macos:** `ffmpeg`, `ffprobe`, `ffplay`
//! + **android:** `ffmpeg`
//! 
//! ##### Sources
//! + **macos**   [luffyxddev/ffbins](https://github.com/luffyxddev/ffbins)
//! + **linux**   [luffyxddev/ffbins](https://github.com/luffyxddev/ffbins)
//! + **windows** [luffyxddev/ffbins](https://github.com/luffyxddev/ffbins)
//! + **android** [luffyxddev/ffbins](https://github.com/luffyxddev/ffbins)
//!
//mod utils;



use std::fs::File;
use std::io::{BufRead, Read, Write};
use std::{env::consts, io::BufReader};
use std::path::PathBuf;





#[derive(Debug, Clone)]
pub enum Version {
    V7_1_2,
    V8_0_1
}

impl Version {
    pub fn to_str(&self) -> &str {
        match self {
            Version::V7_1_2 => "7.1.2",
            Version::V8_0_1 => "8.0.1",
        }
    }
}





#[derive(Debug, Clone)]
pub enum Binary {
    FFmpeg,
    FFprobe,
    FFplay,
}

impl Binary {
    pub fn to_str(&self) -> &str {
        match self {
            Binary::FFmpeg   => "ffmpeg",
            Binary::FFprobe  => "ffprobe",
            Binary::FFplay   => "ffplay",
        }
    }
    pub fn to_os(&self) -> &str {
        match self {
            Binary::FFmpeg   => if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" },
            Binary::FFprobe  => if cfg!(target_os = "windows") { "ffprobe.exe" } else { "ffprobe" },
            Binary::FFplay   => if cfg!(target_os = "windows") { "ffplay.exe" } else { "ffplay" },
        }
    }
}





fn filename(bin: Binary, version: Version) -> String {
    format!("{}-{}-{}-{}.7z", bin.to_str(), version.to_str(), consts::OS, consts::ARCH)
}

fn fileurl(bin: Binary, version: Version) -> String {
    format!("https://github.com/luffyxddev/ffbins/releases/download/v1.1.0/{}", filename(bin, version))
}





#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    NotReady,
    Downloading,
    Downloaded,
    Extracting,
    Extracted,
    Ready,

    WasInstalled,
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
            
            State::WasInstalled => "was_installed".to_string(),
        }
    }
}





#[derive(Default, Debug, Clone)]
struct Progress {
    total: u64,
    current: u64,
    percent: f64,
}





#[derive(Debug, Clone)]
pub struct FFbins {
    bin: Binary,
    version: Version,
    state: State,
    extract: Progress,
    download: Progress,
    temp: Option<PathBuf>,
    dest: Option<PathBuf>,
    binary: Option<PathBuf>,
}

impl FFbins {

    pub fn new(bin: Binary, version: Version, temp: PathBuf, dest: PathBuf) -> Self {
        Self {
            version,
            binary: None,
            temp: Some(temp),
            dest: Some(dest),
            bin: bin.clone(),
            state: State::NotReady,
            extract: Progress::default(),
            download: Progress::default(),
        }
    }

    pub fn init(self) -> Result<Self> {
        std::fs::create_dir_all(self.temp.as_ref().unwrap())?;
        std::fs::create_dir_all(self.dest.as_ref().unwrap())?;
        Ok(self)
    }

    pub fn binary(&self) -> &PathBuf {
        if self.state != State::Ready && self.binary.is_none() {
            Error::Unknown("you need to run .install()".to_string());
        }
        self.binary.as_ref().unwrap()
    }



    fn extract<T>(&mut self, callback: T) -> Result<()>
    where
        Self: Clone + Sized,
        T: Fn(State, u64, u64, f64) + Send + Sync,
    {

        let source = self.temp.as_ref().unwrap().join(filename(self.bin.clone(), self.version.clone()));
        let mut archive = sevenz_rust2::ArchiveReader::open(source, sevenz_rust2::Password::empty())?;
        self.extract.total =  archive.archive().files.iter().filter(|e| e.has_stream()).map(|e| e.size()).sum::<u64>();

        archive.for_each_entries(|entry, reader| {
            let mut buf = [0u8; 8192];
            let path = self.dest.as_ref().unwrap().join(entry.name());
            std::fs::create_dir_all(path.parent().unwrap())?;
            let mut file = File::create(path)?;
            loop {
                let read_size = reader.read(&mut buf)?;
                if read_size == 0 {
                    break Ok(true);
                }
                file.write_all(&buf[..read_size])?;
                self.extract.current += read_size as u64;
                self.extract.percent = (self.extract.current as f64 / self.extract.total as f64) * 100f64;
                callback(self.state.clone(), self.extract.current, self.extract.total, self.extract.percent);
            }
        })?;

        self.state = State::Extracted;
        callback(self.state.clone(), self.extract.current, self.extract.total, self.extract.percent);

        Ok(())
    }

    fn download<T>(&mut self, callback: T) -> Result<()>
    where
        Self: Clone + Sized,
        T: Fn(State, u64, u64, f64) + Send + Sync,
    {
        let mut response = reqwest::blocking::get(fileurl(self.bin.clone(), self.version.clone()))?;
        let mut file = File::create(self.temp.as_ref().unwrap().join(filename(self.bin.clone(), self.version.clone())))?;
        self.download.total = response.content_length().unwrap();
        let mut buffer = [0; 8192];

        loop {
            let n = response.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            file.write_all(&buffer[..n])?;
            self.download.current += n as u64;
            self.download.percent = self.download.current as f64 / self.download.total as f64 * 100f64;
            self.state = State::Downloading;
            if self.download.percent < 100f64 {
                callback(self.state.clone(), self.download.current, self.download.total, self.download.percent);
            }
        }

        self.state = State::Downloaded;
        callback(self.state.clone(), self.download.current, self.download.total, self.download.percent);

        Ok(())
    }

    pub fn install<T>(&mut self, callback: T) -> Result<()>
    where
        Self: Clone + Sized,
        T: Fn(State, u64, u64, f64) + Send + Sync,
    {
        let bin_name = self.bin.clone();

        let mut bins: Vec<String> = vec![];
        for item in which::which_all_global(bin_name.to_str())? {
            bins.push(item.display().to_string());
            self.state = State::WasInstalled;
        }

        callback(self.state.clone(), 0, 0, 0.0);

        if bins.len() == 0 {
            self.download(&callback)?;
            self.extract(&callback)?;
            bins.push(self.dest.as_ref().unwrap().join(bin_name.to_os()).display().to_string());
        }

        self.binary = Some(PathBuf::from(bins.first().unwrap()));

        self.state = State::Ready;

        callback(self.state.clone(), 0, 0, 0.0);

        Ok(())

    }

}





#[allow(unused)]
#[derive(Debug, Default, serde::Deserialize)]
pub struct Process {
    speed: String,
    total_size: String,
    progress: String,
}





pub struct FFbinsCommands {
    binary: PathBuf,
    command: Vec<String>,
}

impl FFbinsCommands {

    pub fn new<B>(binary: B) -> Self
    where 
        B: Into<PathBuf>,
    {
        Self { 
            binary: binary.into(),
            command: vec![ "-progress".to_string(), "pipe:1".to_string(), "-i".to_string() ],
        }
    }

    pub fn command(mut self, input: PathBuf) -> Self {
        self.command.push(input.display().to_string());
        self
    }

    pub fn output(mut self, output: PathBuf) -> Self {
        self.command.push(output.display().to_string());
        self
    }

    pub fn args<I>(mut self, args: Vec<String>) -> Self {
        self.command.extend(args);
        self
    }

    pub fn spawn(&mut self, callback: fn(String, String)) -> Result<()> {

        let mut child = std::process::Command::new(self.binary.clone())
            .args(self.command.clone())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        
        let stdout = child.stdout.take().expect("stderr was not piped");
        let stderr = child.stderr.take().expect("stderr was not piped");

        

        let stdout_handle = std::thread::spawn(move || {

            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                match line {
                    Ok(line) => {

                        let parts: Vec<&str> = line.split("=").collect();

                        callback(parts[0].to_string(), parts[1].trim().to_string());

                    },
                    Err(e)  => eprintln!("stderr error: {}", e),
                }
            }
        });

        let stderr_handle = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(_line) => {}, //callback(format!("stderr error: {}", line)),
                    Err(e)  => eprintln!("stderr error: {}", e),
                }
            }
        });

        // Wait for process to finish
        let status = child.wait()?;
        println!("Process exited with: {:?}", status);

        // Clean up threads
        stdout_handle.join().unwrap();
        stderr_handle.join().unwrap();

        Ok(())
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
    JoinPathsError(#[from] std::env::JoinPathsError),
    
    // std errors
    #[error(transparent)]
    WhichError(#[from] which::Error),
    
    // std errors
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    
    // std errors
    #[error(transparent)]
    SevenZipError(#[from] sevenz_rust2::Error),

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