use ffbins_rs::{FFbins, Options, Binary, Versions};


fn main() {


    let cwd = std::env::current_dir().unwrap().join(".temp");

    let mut cmd = FFbins::new(Options {
        dest: cwd.join("dest"),
        temp: cwd.join("temp"),
        binary: Binary::FFmpeg,
        version: Versions::V7_1
    });
    
    cmd.init().unwrap();
    
}