mod latex;

use tauri::Manager;

#[tauri::command]
fn available_latex_compilers() -> Vec<latex::LatexCompiler> {
    latex::available_compilers()
}

#[tauri::command]
fn compile_latex_document(
    app: tauri::AppHandle,
    request: latex::CompileLatexDocumentRequest,
) -> latex::CompileLatexDocumentResult {
    match app.path().app_cache_dir() {
        Ok(cache_dir) => latex::compile_document(request, &cache_dir.join("compile-runs")),
        Err(error) => latex::CompileLatexDocumentResult {
            success: false,
            log: error.to_string(),
            pdf_path: None,
        },
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            available_latex_compilers,
            compile_latex_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{available_latex_compilers, latex};

    #[test]
    fn exposes_supported_latex_compilers() {
        let compilers = available_latex_compilers();

        assert_eq!(compilers.len(), 4);
        assert_eq!(compilers[0].id, "pdflatex");
        assert_eq!(compilers[0].label, "pdfLaTeX");
        assert!(compilers[0].is_default);
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
        assert!(!compilers[3].is_default);
    }
}
