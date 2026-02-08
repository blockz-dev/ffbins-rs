use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use serde::Deserialize;
use tar::Archive;
use xz2::read::XzDecoder;
use zip::read::ZipArchive;
use sevenz_rust::decompress_file_with_extract_fn;





pub enum ArchiveType {
    Zip,
    SevenZip,
    TarXz,
}

impl ArchiveType {
    pub fn content_type(&self) -> String {
        match self {
            ArchiveType::Zip => "application/zip".to_string(),
            ArchiveType::SevenZip => "application/x-7z-compressed".to_string(),
            ArchiveType::TarXz => "application/x-xz".to_string(), // Using what GitHub uses
        }
    }

    pub fn from_filename(filename: &str) -> crate::Result<Self> {
        if filename.ends_with(".zip") {
            Ok(ArchiveType::Zip)
        }
        else if filename.ends_with(".7z") {
            Ok(ArchiveType::SevenZip)
        }
        else if filename.ends_with(".tar.xz") {
            Ok(ArchiveType::TarXz)
        }
        else {
            Err(crate::Error::UnsupportedArchiveType(filename.to_string()))
        }
    }
}








#[derive(Clone, Deserialize)]
pub struct ExtractorProgress {
    pub name: String,
    pub current: u64,
    pub total: u64,
    pub percent: f64,
}

pub struct Extractor {
    input: String,
    output: String,
    content_type: String,
}

impl Extractor {
    fn new(input: String, output: String) -> Self {
        Self {
            input,
            output,
            content_type : String::new(),
        }
    }

    pub fn load(input: String, output: String) -> crate::Result<Self> {
        let mut this = Extractor::new(input, output);
        this.content_type = ArchiveType::from_filename(&this.input)?.content_type();
        Ok(this)
    }

    // progrees with callback fn
    pub fn extract<'a, T>(&self, callback: T) -> crate::Result<()>
    where 
        T: FnMut(ExtractorProgress) + Send + Sync + 'a,
    {
        match ArchiveType::from_filename(self.input.as_str())? {
            ArchiveType::Zip => self.extract_zip(callback),
            ArchiveType::SevenZip => self.extract_7z(callback),
            ArchiveType::TarXz => self.extract_tar_xz(callback),
        }
    }



    fn extract_zip<'a, T>(&self, mut callback: T) -> crate::Result<()>
    where 
        T: FnMut(ExtractorProgress) + Send + Sync + 'a,
    {

        let file = File::open(&self.input)?;

        let mut archive = ZipArchive::new(BufReader::new(file))?;

        let total:  u64 = archive.len() as u64;
        let mut number: u64 = 0;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = Path::new(&self.output).join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
            number += 1;

            println!("{}: {}% complete", self.input, (number * 100) / total);

            let data = ExtractorProgress {
                name:  file.name().to_string(),
                current:  file.size(),
                total: total,
                percent: (number * 100) as f64 / total as f64
            };

            callback(data);
        }

        Ok(())

    }

    fn extract_7z<'a, T>(&self, mut callback: T) -> crate::Result<()>
    where 
        T: FnMut(ExtractorProgress) + Send + Sync + 'a,
    {
        decompress_file_with_extract_fn(Path::new(&self.input), Path::new(&self.output), |entry, _data, file| {

            println!("{}: {}% complete", self.input, (file.metadata().unwrap().len() * 100) / entry.size());

            callback(ExtractorProgress {
                name:  entry.name().to_string(),
                current:  entry.size(),
                total: entry.size(),
                percent: (file.metadata().unwrap().len() * 100) as f64 / entry.size() as f64
            });

            Ok(true)

        })?;
        Ok(())
    }

    fn extract_tar_xz<'a, T>(&self, mut callback: T) -> crate::Result<()>
    where 
        T: FnMut(ExtractorProgress) + Send + Sync + 'a,
    {
        let tar_xz = File::open(&self.input)?;
        let tar = XzDecoder::new(BufReader::new(tar_xz));
        let mut archive = Archive::new(tar);
        let entries = archive.entries()?;

        let total:  u64 = entries.count() as u64;
        let mut number: u64 = 0;



        for entry in archive.entries()? {
            let mut entry = entry?;
            entry.unpack_in(&self.output)?;

            number += 1;

            println!("{}: {}% complete", self.input, (number * 100) / total);

            callback(ExtractorProgress {
                name:  entry.header().link_name().unwrap().unwrap().to_string_lossy().to_string(),
                current:  entry.header().size().unwrap(),
                total: total,
                percent: (number * 100) as f64 / total as f64
            });
        }
        
        Ok(())
    }
}
