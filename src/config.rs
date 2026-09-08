use base64::{Engine as _, engine::general_purpose::STANDARD};
use miniserde::{Deserialize, json};
use std::{error::Error, fmt::Write as _, fs, io, path::Path};

use crate::format;

#[derive(Deserialize)]
pub struct Config {
    pub bind: String,
    pub proxy_protocol_v2: Option<bool>,
    version_name: String,
    motd: Vec<String>,
    hover_list: Vec<String>,
    server_icon: Option<String>,
    disconnect: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind: "0.0.0.0:25565".into(),
            proxy_protocol_v2: Some(true),
            version_name: "&cMaintenance".into(),
            motd: vec![
                "&eServer unavailable".into(),
                "&7Please try again later.".into(),
            ],
            hover_list: vec!["&cServer is under maintenance".into()],
            server_icon: None,
            disconnect: vec![
                "&b&lDisconnected".into(),
                String::new(),
                "&7Server is under maintenance. Please try again later.".into(),
            ],
        }
    }
}

pub struct Messages {
    pub proxy_protocol_v2: bool,
    pub status_prefix: String,
    pub status_suffix: String,
    pub disconnect_json: String,
}

impl Messages {
    pub fn from_config(config: Config) -> io::Result<Self> {
        let version_name = json::to_string(&format::string(&config.version_name));
        let motd = format::component(&config.motd.join("\n"));
        let sample = player_sample(&config.hover_list);
        let favicon = icon_data(config.server_icon.as_deref())?
            .map(|icon| format!(r#","favicon":{}"#, json::to_string(&icon)))
            .unwrap_or_default();
        let disconnect_json = format::component(&config.disconnect.join("\n"));
        let status_prefix = format!(r#"{{"version":{{"name":{version_name},"protocol":"#);
        let status_suffix = format!(
            r#"}},"players":{{"max":0,"online":0,"sample":{sample}}},"description":{motd}{favicon}}}"#,
        );

        Ok(Self {
            proxy_protocol_v2: config.proxy_protocol_v2.unwrap_or(true),
            status_prefix,
            status_suffix,
            disconnect_json,
        })
    }
}

fn player_sample(players: &[String]) -> String {
    let mut sample = String::from("[");
    for (index, player) in players.iter().enumerate() {
        if index != 0 {
            sample.push(',');
        }
        let name = json::to_string(&format::string(player));
        let id = format!("00000000-0000-0000-0000-{:012x}", index + 1);
        write!(sample, r#"{{"name":{name},"id":"{id}"}}"#).unwrap();
    }
    sample.push(']');
    sample
}

fn icon_data(source: Option<&str>) -> io::Result<Option<String>> {
    let Some(source) = source else {
        return Ok(None);
    };
    if source.starts_with("data:image/png;base64,") {
        return Ok(Some(source.to_owned()));
    }

    let bytes = fs::read(source)?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err(invalid("server_icon must be a PNG file or PNG data URI"));
    }
    Ok(Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(bytes)
    )))
}

pub fn load(path: impl AsRef<Path>) -> Result<Config, Box<dyn Error>> {
    let path = path.as_ref();
    if !path.exists() {
        fs::write(path, include_str!("../mirage.json"))?;
    }
    Ok(json::from_str(&fs::read_to_string(path)?)?)
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
