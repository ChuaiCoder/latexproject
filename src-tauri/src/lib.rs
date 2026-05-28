mod latex;

#[tauri::command]
fn available_latex_engines() -> Vec<latex::LatexEngine> {
    latex::available_engines()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![available_latex_engines])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{available_latex_engines, latex};

    #[test]
    fn exposes_supported_latex_engines() {
        let engines = available_latex_engines();

        assert_eq!(engines.len(), 2);
        assert_eq!(engines[0].id, "miktex");
        assert_eq!(engines[0].label, "MiKTeX");
        assert!(engines[0].is_default);
        assert!(matches!(
            engines[0].status,
            latex::LatexEngineStatus::Installed | latex::LatexEngineStatus::Missing
        ));
        assert!(matches!(
            engines[0].status_reason,
            latex::LatexEngineStatusReason::Available
                | latex::LatexEngineStatusReason::NotFound
                | latex::LatexEngineStatusReason::Failed
        ));
        assert_eq!(engines[1].id, "tectonic");
        assert_eq!(engines[1].label, "Tectonic");
        assert!(!engines[1].is_default);
        assert!(matches!(
            engines[1].status,
            latex::LatexEngineStatus::Installed | latex::LatexEngineStatus::Missing
        ));
        assert!(matches!(
            engines[1].status_reason,
            latex::LatexEngineStatusReason::Available
                | latex::LatexEngineStatusReason::NotFound
                | latex::LatexEngineStatusReason::Failed
        ));
    }
}
