use ffbins_rs::{Binary, FFbins, FFbinsCommands, Version};


fn main() -> anyhow::Result<()> {

    let cwd = dirs::data_local_dir().unwrap().join(env!("CARGO_PKG_NAME"));

    let mut ffbins = FFbins::new(Binary::FFmpeg, Version::V8_0_1, cwd.join("temp"), cwd.join("dest")).init().unwrap();

    ffbins.install(|state, current, total, percent| {

        println!("[{}] {}/{} {:.2}%", state.to_string(), current, total, percent);

    })?;


    FFbinsCommands::new(ffbins.binary())
        .command(cwd.join("data").join("1.mp4"))
        .output(cwd.join("data").join("1o.mp3"))
        .spawn(|key, value| {

            println!("{} : {}\n", key, value);

        })?;
    
    Ok(())

}