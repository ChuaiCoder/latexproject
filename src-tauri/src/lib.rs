mod latex;

#[tauri::command]
fn available_latex_engines() -> Vec<&'static str> {
    latex::available_engine_ids()
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
    use super::available_latex_engines;

    #[test]
    fn exposes_supported_latex_engines() {
        assert_eq!(available_latex_engines(), vec!["miktex", "tectonic"]);
    }
}

