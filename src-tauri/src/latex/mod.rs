use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexEngineStatus {
    Installed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexEngineStatusReason {
    Available,
    NotFound,
    Failed,
    Timeout,
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

struct LatexEngineDefinition {
    id: &'static str,
    label: &'static str,
    is_default: bool,
    command: &'static str,
}

const LATEX_ENGINE_DEFINITIONS: &[LatexEngineDefinition] = &[
    LatexEngineDefinition {
        id: "miktex",
        label: "MiKTeX",
        is_default: true,
        command: "miktex",
    },
    LatexEngineDefinition {
        id: "tectonic",
        label: "Tectonic",
        is_default: false,
        command: "tectonic",
    },
];

pub fn available_engines() -> Vec<LatexEngine> {
    detect_engines(LATEX_ENGINE_DEFINITIONS)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EngineDetection {
    status: LatexEngineStatus,
    reason: LatexEngineStatusReason,
}

fn detect_engines(definitions: &'static [LatexEngineDefinition]) -> Vec<LatexEngine> {
    detect_engines_with(definitions, detect_engine)
}

fn detect_engines_with(
    definitions: &'static [LatexEngineDefinition],
    detect: impl Fn(&str) -> EngineDetection + Copy + Send + Sync,
) -> Vec<LatexEngine> {
    thread::scope(|scope| {
        let detection_handles = definitions
            .iter()
            .map(|definition| {
                scope.spawn(move || {
                    let detection = detect(definition.command);

                    LatexEngine {
                        id: definition.id,
                        label: definition.label,
                        is_default: definition.is_default,
                        status: detection.status,
                        status_reason: detection.reason,
                    }
                })
            })
            .collect::<Vec<_>>();

        detection_handles
            .into_iter()
            .map(|handle| handle.join().expect("LaTeX engine detection panicked"))
            .collect()
    })
}

fn detect_engine(command: &str) -> EngineDetection {
    detect_engine_with_timeout(command, &["--version"], Duration::from_millis(800))
}

fn detect_engine_with_timeout(command: &str, args: &[&str], timeout: Duration) -> EngineDetection {
    let mut child = match Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return EngineDetection {
                status: LatexEngineStatus::Missing,
                reason: LatexEngineStatusReason::NotFound,
            };
        }
        Err(_) => {
            return EngineDetection {
                status: LatexEngineStatus::Missing,
                reason: LatexEngineStatusReason::Failed,
            };
        }
    };

    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                return EngineDetection {
                    status: LatexEngineStatus::Installed,
                    reason: LatexEngineStatusReason::Available,
                };
            }
            Ok(Some(_)) => {
                return EngineDetection {
                    status: LatexEngineStatus::Missing,
                    reason: LatexEngineStatusReason::Failed,
                };
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return EngineDetection {
                    status: LatexEngineStatus::Missing,
                    reason: LatexEngineStatusReason::Timeout,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return EngineDetection {
                    status: LatexEngineStatus::Missing,
                    reason: LatexEngineStatusReason::Failed,
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_engine_with_timeout, detect_engines_with, EngineDetection, LatexEngineDefinition,
        LatexEngineStatus, LatexEngineStatusReason,
    };
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn marks_engine_detection_as_missing_when_version_check_times_out() {
        let started_at = Instant::now();
        let detection = detect_engine_with_timeout(
            slow_command(),
            slow_command_args(),
            Duration::from_millis(50),
        );

        assert_eq!(detection.status, LatexEngineStatus::Missing);
        assert_eq!(detection.reason, LatexEngineStatusReason::Timeout);
        assert!(started_at.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn detects_latex_engines_concurrently() {
        static DEFINITIONS: &[LatexEngineDefinition] = &[
            LatexEngineDefinition {
                id: "slow-a",
                label: "Slow A",
                is_default: true,
                command: "slow-a",
            },
            LatexEngineDefinition {
                id: "slow-b",
                label: "Slow B",
                is_default: false,
                command: "slow-b",
            },
        ];

        let started_at = Instant::now();
        let engines = detect_engines_with(DEFINITIONS, |_| {
            thread::sleep(Duration::from_millis(200));
            EngineDetection {
                status: LatexEngineStatus::Installed,
                reason: LatexEngineStatusReason::Available,
            }
        });

        assert_eq!(engines.len(), 2);
        assert!(started_at.elapsed() < Duration::from_millis(350));
    }

    #[cfg(windows)]
    fn slow_command() -> &'static str {
        "powershell"
    }

    #[cfg(windows)]
    fn slow_command_args() -> &'static [&'static str] {
        &["-NoProfile", "-Command", "Start-Sleep -Seconds 2"]
    }

    #[cfg(not(windows))]
    fn slow_command() -> &'static str {
        "sh"
    }

    #[cfg(not(windows))]
    fn slow_command_args() -> &'static [&'static str] {
        &["-c", "sleep 2"]
    }
}
