use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Crash {
    pub reason: String,
}

#[derive(Deserialize, Debug)]
pub struct Timers {
    pub calendar: String,
    pub seconds: u32,
    pub ticks: u32,
}

#[derive(Deserialize, Debug)]
pub struct Game {
    pub gamelog: Vec<String>,
    pub settings_changed: HashMap<String, String>,
    pub timers: Timers,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub blitter: String,
    pub graphics_set: String,
    pub music_driver: String,
    pub music_set: String,
    pub network: String,
    pub sound_driver: String,
    pub sound_set: String,
    pub video_driver: String,
    pub video_info: String,
}

#[derive(Deserialize, Debug)]
pub struct OpenTTDVersion {
    pub content: String,
    pub hash: String,
    pub modified: u32,
    pub newgrf: String,
    pub revision: String,
    pub tagged: u32,
}

#[derive(Deserialize, Debug)]
pub struct OpenTTD {
    pub bits: u32,
    pub build_date: String,
    pub dedicated_build: String,
    pub endian: String,
    pub version: OpenTTDVersion,
}

#[derive(Deserialize, Debug)]
pub struct OS {
    pub hardware_concurrency: u32,
    pub memory: String,
    pub os: String,
    pub release: String,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub configuration: Configuration,
    pub openttd: OpenTTD,
    pub os: OS,
}

#[derive(Deserialize, Debug)]
pub struct CrashLog {
    pub crash: Crash,
    pub date: String,
    pub game: Game,
    pub info: Info,
    pub stacktrace: Vec<String>,
}
