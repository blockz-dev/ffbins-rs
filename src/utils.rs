use std::time::Duration;
use std::str::FromStr;
use std::env::consts;




pub fn http_client<T>(url: &str) -> crate::Result<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    let json : T = reqwest::blocking::ClientBuilder::new()
        .user_agent(
            format!(
                "{}/{} ({}; {}; {})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                consts::OS,
                consts::ARCH,
                consts::FAMILY
            )
        )
        .build()?
        .get(url)
        .send()?
        .json()?;
    Ok(json)
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


