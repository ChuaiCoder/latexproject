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
pub enum LatexEngineStatusReason {
    Available,
    NotFound,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexEngine {
    pub id: &'static str,
    pub label: &'static str,
    pub is_default: bool,
    pub status: LatexEngineStatus,
    pub status_reason: LatexEngineStatusReason,
}

pub fn available_engines() -> Vec<LatexEngine> {
    let miktex_detection = detect_engine("miktex");
    let tectonic_detection = detect_engine("tectonic");

    vec![
        LatexEngine {
            id: "miktex",
            label: "MiKTeX",
            is_default: true,
            status: miktex_detection.status,
            status_reason: miktex_detection.reason,
        },
        LatexEngine {
            id: "tectonic",
            label: "Tectonic",
            is_default: false,
            status: tectonic_detection.status,
            status_reason: tectonic_detection.reason,
        },
    ]
}

struct EngineDetection {
    status: LatexEngineStatus,
    reason: LatexEngineStatusReason,
}

fn detect_engine(command: &str) -> EngineDetection {
    match Command::new(command).arg("--version").output() {
        Ok(output) if output.status.success() => EngineDetection {
            status: LatexEngineStatus::Installed,
            reason: LatexEngineStatusReason::Available,
        },
        Ok(_) => EngineDetection {
            status: LatexEngineStatus::Missing,
            reason: LatexEngineStatusReason::Failed,
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => EngineDetection {
            status: LatexEngineStatus::Missing,
            reason: LatexEngineStatusReason::NotFound,
        },
        Err(_) => EngineDetection {
            status: LatexEngineStatus::Missing,
            reason: LatexEngineStatusReason::Failed,
        },
    }
}
