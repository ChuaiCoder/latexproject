use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ManagedLatexToolchainStatus {
    Installed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedLatexToolchain {
    pub id: &'static str,
    pub label: &'static str,
    pub install_dir: String,
    pub executable_path: String,
    pub compiler_ids: &'static [&'static str],
    pub status: ManagedLatexToolchainStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexDependencyState {
    pub toolchains_dir: String,
    pub packages_dir: String,
    pub managed_toolchains: Vec<ManagedLatexToolchain>,
}

struct LatexCompilerDefinition {
    id: &'static str,
    label: &'static str,
    command: &'static str,
    args: &'static [&'static str],
    managed_toolchain_id: Option<&'static str>,
}

const LATEX_COMPILER_DEFINITIONS: &[LatexCompilerDefinition] = &[
    LatexCompilerDefinition {
        id: "pdflatex",
        label: "pdfLaTeX",
        command: "pdflatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
        managed_toolchain_id: None,
    },
    LatexCompilerDefinition {
        id: "xelatex",
        label: "XeLaTeX",
        command: "xelatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
        managed_toolchain_id: None,
    },
    LatexCompilerDefinition {
        id: "lualatex",
        label: "LuaLaTeX",
        command: "lualatex",
        args: &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
        managed_toolchain_id: None,
    },
    LatexCompilerDefinition {
        id: "tectonic",
        label: "Tectonic",
        command: "tectonic",
        args: &["main.tex"],
        managed_toolchain_id: Some("tectonic"),
    },
];
const COMPILE_RUN_RETENTION_LIMIT: usize = 5;
const MAX_SOURCE_BYTES: usize = 1024 * 1024;
static COMPILE_LOCK: Mutex<()> = Mutex::new(());

pub fn available_compilers(app_data_dir: Option<&Path>) -> Vec<LatexCompiler> {
    detect_compilers(LATEX_COMPILER_DEFINITIONS, app_data_dir)
}

pub fn dependency_state(app_data_dir: &Path) -> Result<LatexDependencyState, String> {
    let toolchains_dir = dependency_toolchains_dir(app_data_dir);
    let packages_dir = dependency_packages_dir(app_data_dir);

    fs::create_dir_all(&toolchains_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&packages_dir).map_err(|error| error.to_string())?;

    Ok(LatexDependencyState {
        toolchains_dir: toolchains_dir.to_string_lossy().into_owned(),
        packages_dir: packages_dir.to_string_lossy().into_owned(),
        managed_toolchains: vec![managed_tectonic_toolchain(&toolchains_dir)],
    })
}

pub fn compile_document(
    request: CompileLatexDocumentRequest,
    cache_root: &Path,
    app_data_dir: Option<&Path>,
) -> CompileLatexDocumentResult {
    with_compile_lock(|| {
        compile_document_for_definitions(
            request,
            cache_root,
            app_data_dir,
            LATEX_COMPILER_DEFINITIONS,
        )
    })
}

fn with_compile_lock<T>(compile: impl FnOnce() -> T) -> T {
    let _guard = COMPILE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    compile()
}

fn compile_document_for_definitions(
    request: CompileLatexDocumentRequest,
    cache_root: &Path,
    app_data_dir: Option<&Path>,
    definitions: &[LatexCompilerDefinition],
) -> CompileLatexDocumentResult {
    if request.source.len() > MAX_SOURCE_BYTES {
        return failed_compile_result(format!(
            "Source is too large. Maximum supported size is {} bytes.",
            MAX_SOURCE_BYTES
        ));
    }

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
    let command = compiler_command(compiler, app_data_dir);

    match compile_document_with(
        request.compiler_id.as_str(),
        request.source.as_str(),
        &working_dir,
        command.as_str(),
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

fn detect_compilers(
    definitions: &'static [LatexCompilerDefinition],
    app_data_dir: Option<&Path>,
) -> Vec<LatexCompiler> {
    detect_compilers_with(definitions, |definition| {
        let command = compiler_command(definition, app_data_dir);
        detect_compiler(command.as_str())
    })
}

fn detect_compilers_with(
    definitions: &'static [LatexCompilerDefinition],
    detect: impl Fn(&LatexCompilerDefinition) -> EngineDetection + Copy + Send + Sync,
) -> Vec<LatexCompiler> {
    thread::scope(|scope| {
        let detection_handles = definitions
            .iter()
            .map(|definition| {
                scope.spawn(move || {
                    let detection = detect(definition);

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
    let working_dir = unique_working_dir_in(cache_root)?;
    fs::create_dir_all(&working_dir).map_err(|error| error.to_string())?;
    prune_compile_runs(cache_root, COMPILE_RUN_RETENTION_LIMIT)?;
    Ok(working_dir)
}

fn prune_compile_runs(cache_root: &Path, retention_limit: usize) -> Result<(), String> {
    let mut compile_runs = Vec::new();

    for entry in fs::read_dir(cache_root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let is_compile_run = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("run-"));

        if path.is_dir() && is_compile_run {
            let modified_at = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(UNIX_EPOCH);
            let file_name = entry.file_name();
            compile_runs.push((modified_at, file_name, path));
        }
    }

    compile_runs.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    let runs_to_remove = compile_runs.len().saturating_sub(retention_limit);
    for (_, _, path) in compile_runs.into_iter().take(runs_to_remove) {
        fs::remove_dir_all(&path).map_err(|error| error.to_string())?;
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

fn compiler_command(definition: &LatexCompilerDefinition, app_data_dir: Option<&Path>) -> String {
    if let (Some(app_data_dir), Some("tectonic")) = (app_data_dir, definition.managed_toolchain_id)
    {
        let toolchains_dir = dependency_toolchains_dir(app_data_dir);
        let managed_toolchain = managed_tectonic_toolchain(&toolchains_dir);

        if managed_toolchain.status == ManagedLatexToolchainStatus::Installed {
            return managed_toolchain.executable_path;
        }
    }

    definition.command.to_string()
}

fn managed_tectonic_toolchain(toolchains_dir: &Path) -> ManagedLatexToolchain {
    let install_dir = toolchains_dir.join("tectonic");
    let executable_path = install_dir.join(executable_name("tectonic"));

    ManagedLatexToolchain {
        id: "tectonic",
        label: "Tectonic",
        install_dir: install_dir.to_string_lossy().into_owned(),
        executable_path: executable_path.to_string_lossy().into_owned(),
        compiler_ids: &["tectonic"],
        status: if executable_path.exists() {
            ManagedLatexToolchainStatus::Installed
        } else {
            ManagedLatexToolchainStatus::Missing
        },
    }
}

fn dependency_toolchains_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("dependencies").join("toolchains")
}

fn dependency_packages_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("dependencies").join("packages")
}

fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compile_document_with, compiler_command, dependency_state, detect_compiler_with_timeout,
        detect_compilers_with, executable_name, prepare_working_dir, CompileLatexDocumentRequest,
        EngineDetection, LatexCompilerDefinition, LatexCompilerStatus, LatexCompilerStatusReason,
        ManagedLatexToolchainStatus,
    };
    use std::fs;
    use std::path::Path;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
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
                managed_toolchain_id: None,
            },
            LatexCompilerDefinition {
                id: "slow-b",
                label: "Slow B",
                command: "slow-b",
                args: &["main.tex"],
                managed_toolchain_id: None,
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
                managed_toolchain_id: None,
            },
            LatexCompilerDefinition {
                id: "installed-fallback",
                label: "Installed Fallback",
                command: "installed-fallback",
                args: &["main.tex"],
                managed_toolchain_id: None,
            },
        ];

        let compilers = detect_compilers_with(DEFINITIONS, |definition| {
            if definition.command == "installed-fallback" {
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
        let compilers = super::available_compilers(None);

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
    fn managed_toolchain_executable_takes_precedence_over_path_command() {
        let app_data_dir = unique_temp_dir("managed-tectonic-command");
        let managed_executable = app_data_dir
            .join("dependencies")
            .join("toolchains")
            .join("tectonic")
            .join(executable_name("tectonic"));
        fs::create_dir_all(
            managed_executable
                .parent()
                .expect("managed executable should have parent"),
        )
        .expect("managed executable dir should be created");
        fs::write(&managed_executable, "").expect("managed executable should be written");
        let definition = LatexCompilerDefinition {
            id: "tectonic",
            label: "Tectonic",
            command: "tectonic",
            args: &["main.tex"],
            managed_toolchain_id: Some("tectonic"),
        };

        let command = compiler_command(&definition, Some(&app_data_dir));

        assert_eq!(command, managed_executable.to_string_lossy());

        let _ = fs::remove_dir_all(app_data_dir);
    }

    #[test]
    fn dependency_state_creates_app_managed_latex_directories() {
        let app_data_dir = unique_temp_dir("dependency-state");

        let state = dependency_state(&app_data_dir).expect("dependency state should load");

        assert!(Path::new(&state.toolchains_dir).exists());
        assert!(Path::new(&state.packages_dir).exists());
        assert!(
            state.toolchains_dir.ends_with("dependencies\\toolchains")
                || state.toolchains_dir.ends_with("dependencies/toolchains")
        );
        assert_eq!(state.managed_toolchains.len(), 1);
        assert_eq!(state.managed_toolchains[0].id, "tectonic");
        assert_eq!(state.managed_toolchains[0].compiler_ids, &["tectonic"]);
        assert_eq!(
            state.managed_toolchains[0].status,
            ManagedLatexToolchainStatus::Missing
        );

        let _ = fs::remove_dir_all(app_data_dir);
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
            managed_toolchain_id: None,
        }];
        let cache_root = unique_temp_dir("spawn-failure");

        let result = super::compile_document_for_definitions(
            CompileLatexDocumentRequest {
                compiler_id: "missing-compiler".to_string(),
                source: "\\documentclass{article}\\begin{document}Hi\\end{document}".to_string(),
            },
            &cache_root,
            None,
            DEFINITIONS,
        );

        assert!(!result.success);
        assert!(result.pdf_path.is_none());
        assert!(!result.log.is_empty());

        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn preserves_existing_compile_run_output_for_preview() {
        let cache_root = unique_temp_dir("compile-cache");
        let previous_run = cache_root.join("run-previous");
        let unrelated_dir = cache_root.join("other");
        fs::create_dir_all(&previous_run).expect("previous run should be created");
        fs::write(previous_run.join("main.pdf"), "PDF").expect("previous PDF should be written");
        fs::create_dir_all(&unrelated_dir).expect("unrelated dir should be created");

        let working_dir = prepare_working_dir(&cache_root).expect("working dir should be prepared");

        assert!(working_dir.starts_with(&cache_root));
        assert!(previous_run.join("main.pdf").exists());
        assert!(unrelated_dir.exists());

        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn prunes_oldest_compile_runs_beyond_retention_limit() {
        let cache_root = unique_temp_dir("compile-cache-retention");
        fs::create_dir_all(&cache_root).expect("cache root should be created");

        let old_run = cache_root.join("run-0");
        fs::create_dir_all(&old_run).expect("old run should be created");
        thread::sleep(Duration::from_millis(20));

        let mut recent_runs = Vec::new();
        for index in 1..=5 {
            let run = cache_root.join(format!("run-{index}"));
            fs::create_dir_all(&run).expect("recent run should be created");
            fs::write(run.join("main.pdf"), "PDF").expect("recent PDF should be written");
            recent_runs.push(run);
            thread::sleep(Duration::from_millis(20));
        }

        let working_dir = prepare_working_dir(&cache_root).expect("working dir should be prepared");
        let retained_runs = fs::read_dir(&cache_root)
            .expect("cache root should be readable")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().is_dir()
                    && entry
                        .file_name()
                        .to_str()
                        .is_some_and(|name| name.starts_with("run-"))
            })
            .count();

        assert!(working_dir.starts_with(&cache_root));
        assert!(!old_run.exists());
        assert!(recent_runs
            .last()
            .expect("recent runs should exist")
            .join("main.pdf")
            .exists());
        assert_eq!(retained_runs, 5);

        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn compile_document_waits_for_existing_compile_lock() {
        let cache_root = unique_temp_dir("serialized-compiles");
        let returned = Arc::new(AtomicBool::new(false));
        let compile_guard = super::COMPILE_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        thread::scope(|scope| {
            let returned_in_thread = Arc::clone(&returned);
            let cache_root_ref = &cache_root;
            let compile = scope.spawn(move || {
                let result = super::compile_document(
                    CompileLatexDocumentRequest {
                        compiler_id: "unsupported".to_string(),
                        source: "\\documentclass{article}\\begin{document}Hi\\end{document}"
                            .to_string(),
                    },
                    cache_root_ref,
                    None,
                );
                returned_in_thread.store(true, Ordering::SeqCst);
                result
            });

            thread::sleep(Duration::from_millis(50));
            assert!(!returned.load(Ordering::SeqCst));
            drop(compile_guard);

            let result = compile.join().expect("compile should not panic");
            assert!(!result.success);
            assert!(result.log.contains("Unsupported LaTeX compiler"));
        });

        assert!(returned.load(Ordering::SeqCst));

        let _ = fs::remove_dir_all(cache_root);
    }

    #[test]
    fn rejects_source_that_exceeds_compile_request_limit() {
        static DEFINITIONS: &[LatexCompilerDefinition] = &[LatexCompilerDefinition {
            id: "pdflatex",
            label: "pdfLaTeX",
            command: "pdflatex",
            args: &["main.tex"],
            managed_toolchain_id: None,
        }];
        let cache_root = unique_temp_dir("oversized-source");

        let result = super::compile_document_for_definitions(
            CompileLatexDocumentRequest {
                compiler_id: "pdflatex".to_string(),
                source: "x".repeat(super::MAX_SOURCE_BYTES + 1),
            },
            &cache_root,
            None,
            DEFINITIONS,
        );

        assert!(!result.success);
        assert!(result.pdf_path.is_none());
        assert!(result.log.contains("Source is too large"));
        assert!(!cache_root.exists());
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
