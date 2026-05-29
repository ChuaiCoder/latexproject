use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexCompilerStatus {
    Installed,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LatexCompilerStatusReason {
    Available,
    NotFound,
    Failed,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexCompiler {
    pub id: &'static str,
    pub label: &'static str,
    pub is_default: bool,
    pub status: LatexCompilerStatus,
    pub status_reason: LatexCompilerStatusReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileLatexDocumentRequest {
    pub compiler_id: String,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileLatexDocumentResult {
    pub success: bool,
    pub log: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_path: Option<String>,
}

struct LatexCompilerDefinition {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    args: &'static [&'static str],
}

const LATEX_COMPILER_DEFINITIONS: &[LatexCompilerDefinition] = &[
    LatexCompilerDefinition {
        id: "pdflatex",
        label: "pdfLaTeX",
        command: "pdflatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
    },
    LatexCompilerDefinition {
        id: "xelatex",
        label: "XeLaTeX",
        command: "xelatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
    },
    LatexCompilerDefinition {
        id: "lualatex",
        label: "LuaLaTeX",
        command: "lualatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
    },
    LatexCompilerDefinition {
        id: "tectonic",
        label: "Tectonic",
        command: "tectonic",
        args: &["main.tex"],
    },
];

pub fn available_compilers() -> Vec<LatexCompiler> {
    detect_compilers(LATEX_COMPILER_DEFINITIONS)
}

pub fn compile_document(
    request: CompileLatexDocumentRequest,
    cache_root: &Path,
) -> CompileLatexDocumentResult {
    compile_document_for_definitions(request, cache_root, LATEX_COMPILER_DEFINITIONS)
}

fn compile_document_for_definitions(
    request: CompileLatexDocumentRequest,
    cache_root: &Path,
    definitions: &[LatexCompilerDefinition],
) -> CompileLatexDocumentResult {
    let compiler = match definitions
        .iter()
        .find(|compiler| compiler.id == request.compiler_id)
    {
        Some(compiler) => compiler,
        None => {
            return failed_compile_result(format!(
                "Unsupported LaTeX compiler: {}",
                request.compiler_id
            ));
        }
    };
    let working_dir = match prepare_working_dir(cache_root) {
        Ok(working_dir) => working_dir,
        Err(error) => return failed_compile_result(error),
    };

    match compile_document_with(
        request.compiler_id.as_str(),
        request.source.as_str(),
        &working_dir,
        compiler.command,
        compiler.args,
        Duration::from_secs(20),
    ) {
        Ok(result) => result,
        Err(error) => failed_compile_result(error),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EngineDetection {
    status: LatexCompilerStatus,
    reason: LatexCompilerStatusReason,
}

fn detect_compilers(definitions: &'static [LatexCompilerDefinition]) -> Vec<LatexCompiler> {
    detect_compilers_with(definitions, detect_compiler)
}

fn detect_compilers_with(
    definitions: &'static [LatexCompilerDefinition],
    detect: impl Fn(&str) -> EngineDetection + Copy + Send + Sync,
) -> Vec<LatexCompiler> {
    thread::scope(|scope| {
        let detection_handles = definitions
            .iter()
            .map(|definition| {
                scope.spawn(move || {
                    let detection = detect(definition.command);

                    LatexCompiler {
                        id: definition.id,
                        label: definition.label,
                        is_default: false,
                        status: detection.status,
                        status_reason: detection.reason,
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut compilers = detection_handles
            .into_iter()
            .map(|handle| handle.join().expect("LaTeX compiler detection panicked"))
            .collect::<Vec<_>>();

        if let Some(default_compiler) = compilers
            .iter_mut()
            .find(|compiler| compiler.status == LatexCompilerStatus::Installed)
        {
            default_compiler.is_default = true;
        }

        compilers
    })
}

fn detect_compiler(command: &str) -> EngineDetection {
    detect_compiler_with_timeout(command, &["--version"], Duration::from_secs(3))
}

fn detect_compiler_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> EngineDetection {
    let mut child = match Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return EngineDetection {
                status: LatexCompilerStatus::Missing,
                reason: LatexCompilerStatusReason::NotFound,
            };
        }
        Err(_) => {
            return EngineDetection {
                status: LatexCompilerStatus::Missing,
                reason: LatexCompilerStatusReason::Failed,
            };
        }
    };

    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => {
                return EngineDetection {
                    status: LatexCompilerStatus::Installed,
                    reason: LatexCompilerStatusReason::Available,
                };
            }
            Ok(Some(_)) => {
                return EngineDetection {
                    status: LatexCompilerStatus::Missing,
                    reason: LatexCompilerStatusReason::Failed,
                };
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return EngineDetection {
                    status: LatexCompilerStatus::Missing,
                    reason: LatexCompilerStatusReason::Timeout,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return EngineDetection {
                    status: LatexCompilerStatus::Missing,
                    reason: LatexCompilerStatusReason::Failed,
                };
            }
        }
    }
}

fn compile_document_with(
    _compiler_id: &str,
    source: &str,
    working_dir: &Path,
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<CompileLatexDocumentResult, String> {
    fs::create_dir_all(working_dir).map_err(|error| error.to_string())?;
    fs::write(working_dir.join("main.tex"), source).map_err(|error| error.to_string())?;

    let mut child = Command::new(command)
        .args(args)
        .current_dir(working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    let started_at = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                let log = format_process_log(&output.stdout, &output.stderr);
                let pdf_path = working_dir.join("main.pdf");

                return Ok(CompileLatexDocumentResult {
                    success: output.status.success() && pdf_path.exists(),
                    log,
                    pdf_path: pdf_path
                        .exists()
                        .then(|| pdf_path.to_string_lossy().into_owned()),
                });
            }
            Ok(None) if started_at.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child
                    .wait_with_output()
                    .map_err(|error| error.to_string())?;
                let log = format_process_log(&output.stdout, &output.stderr);

                return Ok(CompileLatexDocumentResult {
                    success: false,
                    log: if log.is_empty() {
                        "Compile timed out.".to_string()
                    } else {
                        format!("Compile timed out.\n{log}")
                    },
                    pdf_path: None,
                });
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.to_string());
            }
        }
    }
}

fn format_process_log(stdout: &[u8], stderr: &[u8]) -> String {
    let mut log = String::new();
    log.push_str(&String::from_utf8_lossy(stdout));
    log.push_str(&String::from_utf8_lossy(stderr));
    log
}

fn prepare_working_dir(cache_root: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(cache_root).map_err(|error| error.to_string())?;
    prune_compile_runs(cache_root)?;
    let working_dir = unique_working_dir_in(cache_root)?;
    fs::create_dir_all(&working_dir).map_err(|error| error.to_string())?;
    Ok(working_dir)
}

fn prune_compile_runs(cache_root: &Path) -> Result<(), String> {
    for entry in fs::read_dir(cache_root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let is_compile_run = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("run-"));

        if path.is_dir() && is_compile_run {
            fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn unique_working_dir_in(cache_root: &Path) -> Result<PathBuf, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();

    Ok(cache_root.join(format!("run-{}-{timestamp}", std::process::id())))
}

fn failed_compile_result(log: String) -> CompileLatexDocumentResult {
    CompileLatexDocumentResult {
        success: false,
        log,
        pdf_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compile_document_with, detect_compiler_with_timeout, detect_compilers_with,
        prepare_working_dir, CompileLatexDocumentRequest, EngineDetection, LatexCompilerDefinition,
        LatexCompilerStatus, LatexCompilerStatusReason,
    };
    use std::fs;
    use std::path::Path;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn marks_compiler_detection_as_missing_when_version_check_times_out() {
        let started_at = Instant::now();
        let detection = detect_compiler_with_timeout(
            slow_command(),
            slow_command_args(),
            Duration::from_millis(50),
        );

        assert_eq!(detection.status, LatexCompilerStatus::Missing);
        assert_eq!(detection.reason, LatexCompilerStatusReason::Timeout);
        assert!(started_at.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn detects_latex_compilers_concurrently() {
        static DEFINITIONS: &[LatexCompilerDefinition] = &[
            LatexCompilerDefinition {
                id: "slow-a",
                label: "Slow A",
                command: "slow-a",
                args: &["main.tex"],
            },
            LatexCompilerDefinition {
                id: "slow-b",
                label: "Slow B",
                command: "slow-b",
                args: &["main.tex"],
            },
        ];

        let started_at = Instant::now();
        let compilers = detect_compilers_with(DEFINITIONS, |_| {
            thread::sleep(Duration::from_millis(200));
            EngineDetection {
                status: LatexCompilerStatus::Installed,
                reason: LatexCompilerStatusReason::Available,
            }
        });

        assert_eq!(compilers.len(), 2);
        assert!(started_at.elapsed() < Duration::from_millis(350));
    }

    #[test]
    fn marks_first_installed_compiler_as_default() {
        static DEFINITIONS: &[LatexCompilerDefinition] = &[
            LatexCompilerDefinition {
                id: "missing-default",
                label: "Missing Default",
                command: "missing-default",
                args: &["main.tex"],
            },
            LatexCompilerDefinition {
                id: "installed-fallback",
                label: "Installed Fallback",
                command: "installed-fallback",
                args: &["main.tex"],
            },
        ];

        let compilers = detect_compilers_with(DEFINITIONS, |command| {
            if command == "installed-fallback" {
                EngineDetection {
                    status: LatexCompilerStatus::Installed,
                    reason: LatexCompilerStatusReason::Available,
                }
            } else {
                EngineDetection {
                    status: LatexCompilerStatus::Missing,
                    reason: LatexCompilerStatusReason::NotFound,
                }
            }
        });

        assert!(!compilers[0].is_default);
        assert!(compilers[1].is_default);
    }

    #[test]
    fn exposes_real_latex_compilers_not_distributions() {
        let compilers = super::available_compilers();

        assert_eq!(
            compilers
                .iter()
                .map(|compiler| compiler.id)
                .collect::<Vec<_>>(),
            vec!["pdflatex", "xelatex", "lualatex", "tectonic"]
        );
        assert_eq!(compilers[0].label, "pdfLaTeX");
    }

    #[test]
    fn default_detection_allows_slow_successful_version_checks() {
        let temp_dir = unique_temp_dir("slow-version-success");
        let command = slow_successful_version_probe(&temp_dir);

        let detection =
            super::detect_compiler(command.to_str().expect("command path should be UTF-8"));

        assert_eq!(detection.status, LatexCompilerStatus::Installed);
        assert_eq!(detection.reason, LatexCompilerStatusReason::Available);

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn compiles_document_and_returns_pdf_path_when_compiler_succeeds() {
        let temp_dir = unique_temp_dir("success");

        let result = compile_document_with(
            "pdflatex",
            "\\documentclass{article}\\begin{document}Hi\\end{document}",
            &temp_dir,
            fake_success_compiler(),
            fake_success_args(),
            Duration::from_secs(1),
        )
        .expect("compile request should complete");

        assert!(result.success);
        assert!(result
            .pdf_path
            .expect("expected pdf path")
            .ends_with("main.pdf"));
        assert!(result.log.contains("compiled"));
        assert!(temp_dir.join("main.tex").exists());
        assert!(temp_dir.join("main.pdf").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn returns_compiler_log_when_compile_fails() {
        let temp_dir = unique_temp_dir("failure");

        let result = compile_document_with(
            "pdflatex",
            "\\documentclass{article}\\begin{document}\\bad\\end{document}",
            &temp_dir,
            fake_failure_compiler(),
            fake_failure_args(),
            Duration::from_secs(1),
        )
        .expect("compile request should complete");

        assert!(!result.success);
        assert!(result.pdf_path.is_none());
        assert!(result.log.contains("Undefined control sequence"));

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn returns_structured_result_when_compiler_spawn_fails() {
        static DEFINITIONS: &[LatexCompilerDefinition] = &[LatexCompilerDefinition {
            id: "missing-compiler",
            label: "Missing Compiler",
            command: "latex-workbench-missing-compiler-for-test",
            args: &["main.tex"],
        }];
        let cache_root = unique_temp_dir("spawn-failure");

        let result = super::compile_document_for_definitions(
            CompileLatexDocumentRequest {
                compiler_id: "missing-compiler".to_string(),
                source: "\\documentclass{article}\\begin{document}Hi\\end{document}".to_string(),
            },
            &cache_root,
            DEFINITIONS,
        );

        assert!(!result.success);
        assert!(result.pdf_path.is_none());
        assert!(!result.log.is_empty());

        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn creates_compile_runs_under_cache_root_and_prunes_old_runs() {
        let cache_root = unique_temp_dir("compile-cache");
        let old_run = cache_root.join("run-old");
        let unrelated_dir = cache_root.join("other");
        fs::create_dir_all(&old_run).expect("old run should be created");
        fs::create_dir_all(&unrelated_dir).expect("unrelated dir should be created");

        let working_dir = prepare_working_dir(&cache_root).expect("working dir should be prepared");

        assert!(working_dir.starts_with(&cache_root));
        assert!(!old_run.exists());
        assert!(unrelated_dir.exists());

        let _ = fs::remove_dir_all(cache_root);
    }

    fn unique_temp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "latex-workbench-test-{name}-{}",
            std::process::id()
        ))
    }

    #[cfg(windows)]
    fn slow_successful_version_probe(temp_dir: &Path) -> std::path::PathBuf {
        let command = temp_dir.join("slow-version.cmd");
        fs::create_dir_all(temp_dir).expect("temp dir should be created");
        fs::write(
            &command,
            "@echo off\r\npowershell -NoProfile -Command \"Start-Sleep -Milliseconds 1000\"\r\nexit /b 0\r\n",
        )
        .expect("slow version probe should be written");
        command
    }

    #[cfg(not(windows))]
    fn slow_successful_version_probe(temp_dir: &Path) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let command = temp_dir.join("slow-version");
        fs::create_dir_all(temp_dir).expect("temp dir should be created");
        fs::write(&command, "#!/bin/sh\nsleep 1\nexit 0\n")
            .expect("slow version probe should be written");
        let mut permissions = fs::metadata(&command)
            .expect("slow version probe should exist")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&command, permissions)
            .expect("slow version probe should be executable");
        command
    }

    #[cfg(windows)]
    fn slow_command() -> &'static str {
        "powershell"
    }

    #[cfg(windows)]
    fn slow_command_args() -> &'static [&'static str] {
        &["-NoProfile", "-Command", "Start-Sleep -Seconds 2"]
    }

    #[cfg(windows)]
    fn fake_success_compiler() -> &'static str {
        "powershell"
    }

    #[cfg(windows)]
    fn fake_success_args() -> &'static [&'static str] {
        &[
            "-NoProfile",
            "-Command",
            "Set-Content -LiteralPath main.pdf -Value 'PDF'; Write-Output 'compiled'",
        ]
    }

    #[cfg(windows)]
    fn fake_failure_compiler() -> &'static str {
        "powershell"
    }

    #[cfg(windows)]
    fn fake_failure_args() -> &'static [&'static str] {
        &[
            "-NoProfile",
            "-Command",
            "Write-Output 'Undefined control sequence'; exit 1",
        ]
    }

    #[cfg(not(windows))]
    fn slow_command() -> &'static str {
        "sh"
    }

    #[cfg(not(windows))]
    fn slow_command_args() -> &'static [&'static str] {
        &["-c", "sleep 2"]
    }

    #[cfg(not(windows))]
    fn fake_success_compiler() -> &'static str {
        "sh"
    }

    #[cfg(not(windows))]
    fn fake_success_args() -> &'static [&'static str] {
        &["-c", "printf PDF > main.pdf; echo compiled"]
    }

    #[cfg(not(windows))]
    fn fake_failure_compiler() -> &'static str {
        "sh"
    }

    #[cfg(not(windows))]
    fn fake_failure_args() -> &'static [&'static str] {
        &["-c", "echo Undefined control sequence; exit 1"]
    }
}
