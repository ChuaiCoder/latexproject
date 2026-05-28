use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexEngineStatus {
    Installed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexEngine {
    pub id: &'static str,
    pub label: &'static str,
    pub is_default: bool,
    pub status: LatexEngineStatus,
}

pub fn available_engines() -> Vec<LatexEngine> {
    vec![
        LatexEngine {
            id: "miktex",
            label: "MiKTeX",
            is_default: true,
            status: detect_engine("miktex"),
        },
        LatexEngine {
            id: "tectonic",
            label: "Tectonic",
            is_default: false,
            status: detect_engine("tectonic"),
        },
    ]
}

fn detect_engine(command: &str) -> LatexEngineStatus {
    match Command::new(command).arg("--version").output() {
        Ok(output) if output.status.success() => LatexEngineStatus::Installed,
        _ => LatexEngineStatus::Missing,
    }
}
