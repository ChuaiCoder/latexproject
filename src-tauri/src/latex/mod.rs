use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatexEngine {
    pub id: &'static str,
    pub label: &'static str,
    pub is_default: bool,
}

pub fn available_engines() -> Vec<LatexEngine> {
    vec![
        LatexEngine {
            id: "miktex",
            label: "MiKTeX",
            is_default: true,
        },
        LatexEngine {
            id: "tectonic",
            label: "Tectonic",
            is_default: false,
        },
    ]
}
