import { Play } from "lucide-react";
import { DEFAULT_ENGINE_ID } from "../domain/latexEngine";

export function App() {
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
            <p>Default engine: {DEFAULT_ENGINE_ID}</p>
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

