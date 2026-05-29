mod latex;

use tauri::Manager;

#[tauri::command]
fn available_latex_compilers(app: tauri::AppHandle) -> Vec<latex::LatexCompiler> {
    match app.path().app_data_dir() {
        Ok(app_data_dir) => latex::available_compilers(Some(&app_data_dir)),
        Err(_) => latex::available_compilers(None),
    }
}

#[tauri::command]
fn compile_latex_document(
    app: tauri::AppHandle,
    request: latex::CompileLatexDocumentRequest,
) -> latex::CompileLatexDocumentResult {
    let app_data_dir = app.path().app_data_dir().ok();

    match app.path().app_cache_dir() {
        Ok(cache_dir) => latex::compile_document(
            request,
            &cache_dir.join("compile-runs"),
            app_data_dir.as_deref(),
        ),
        Err(error) => latex::CompileLatexDocumentResult {
            success: false,
            log: error.to_string(),
            pdf_path: None,
        },
    }
}

#[tauri::command]
fn latex_dependency_state(app: tauri::AppHandle) -> Result<latex::LatexDependencyState, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .and_then(|app_data_dir| latex::dependency_state(&app_data_dir))
}

#[tauri::command]
fn install_latex_toolchain(
    app: tauri::AppHandle,
    request: latex::InstallLatexToolchainRequest,
) -> Result<latex::InstallLatexToolchainResult, String> {
    app.path()
        .app_data_dir()
        .map_err(|error| error.to_string())
        .map(|app_data_dir| latex::install_toolchain(request, &app_data_dir))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            available_latex_compilers,
            compile_latex_document,
            latex_dependency_state,
            install_latex_toolchain
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::latex;

    #[test]
    fn exposes_supported_latex_compilers() {
        let compilers = latex::available_compilers(None);

        assert_eq!(compilers.len(), 4);
        assert_eq!(compilers[0].id, "pdflatex");
        assert_eq!(compilers[0].label, "pdfLaTeX");
        assert!(matches!(
            compilers[0].status,
            latex::LatexCompilerStatus::Installed | latex::LatexCompilerStatus::Missing
        ));
        assert!(matches!(
            compilers[0].status_reason,
            latex::LatexCompilerStatusReason::Available
                | latex::LatexCompilerStatusReason::NotFound
                | latex::LatexCompilerStatusReason::Failed
                | latex::LatexCompilerStatusReason::Timeout
        ));
        assert_eq!(compilers[3].id, "tectonic");
        assert_eq!(compilers[3].label, "Tectonic");
    }

    #[test]
    fn marks_first_installed_compiler_as_default_when_available() {
        let compilers = latex::available_compilers(None);
        let default_compilers = compilers
            .iter()
            .filter(|compiler| compiler.is_default)
            .collect::<Vec<_>>();

        if let Some(first_installed_compiler) = compilers
            .iter()
            .find(|compiler| compiler.status == latex::LatexCompilerStatus::Installed)
        {
            assert_eq!(default_compilers, vec![first_installed_compiler]);
        } else {
            assert!(default_compilers.is_empty());
        }
    }
}
