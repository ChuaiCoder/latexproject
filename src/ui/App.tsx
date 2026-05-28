import { useEffect, useState } from "react";
import { Play } from "lucide-react";
import { loadLatexEngines } from "../backend/latexBackend";
import type { LatexEngine } from "../domain/latexEngine";

export function App() {
  const [engines, setEngines] = useState<LatexEngine[]>([]);
  const [engineError, setEngineError] = useState(false);

  useEffect(() => {
    let isMounted = true;

    loadLatexEngines()
      .then((loadedEngines) => {
        if (isMounted) {
          setEngines(loadedEngines);
          setEngineError(false);
        }
      })
      .catch(() => {
        if (isMounted) {
          setEngines([]);
          setEngineError(true);
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const defaultEngine = engines.find((engine) => engine.isDefault);
  const engineSummary = engines.map((engine) => engine.label).join(", ");
  const formatEngineStatus = (engine: LatexEngine) => {
    if (engine.status === "installed") {
      return "Installed";
    }

    if (engine.statusReason === "notFound") {
      return "Missing (not found on PATH)";
    }

    if (engine.statusReason === "failed") {
      return "Missing (version check failed)";
    }

    return "Missing";
  };

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand" aria-label="LaTeX Workbench">
          <span className="brand-icon">T</span>
          <span>LaTeX Workbench</span>
        </div>
        <button className="compile-button" type="button">
          <Play size={16} aria-hidden="true" /> Compile
        </button>
      </header>

      <section className="workspace" aria-label="Workspace">
        <aside className="panel" aria-label="Project files">
          <div className="panel-header">Files</div>
          <div className="file-list">
            <div className="file-item active">main.tex</div>
            <div className="file-item">references.bib</div>
          </div>
        </aside>

        <section className="panel" aria-label="Editor">
          <div className="panel-header">Editor</div>
          <div className="editor-placeholder">
            <p className="muted">Editor adapter pending.</p>
            {engineError ? (
              <p role="status">Unable to load LaTeX engines.</p>
            ) : (
              <>
                <p>Default engine: {defaultEngine?.label ?? "Loading..."}</p>
                {engineSummary ? <p>Available engines: {engineSummary}</p> : null}
                {engines.length > 0 ? (
                  <ul className="engine-status-list" aria-label="LaTeX engine status">
                    {engines.map((engine) => (
                      <li key={engine.id}>
                        {engine.label}: {formatEngineStatus(engine)}
                      </li>
                    ))}
                  </ul>
                ) : null}
              </>
            )}
          </div>
        </section>

        <section className="panel" aria-label="PDF preview">
          <div className="panel-header">Preview</div>
          <div className="preview-placeholder">
            <p className="muted">PDF preview adapter pending.</p>
          </div>
        </section>
      </section>
    </main>
  );
}
