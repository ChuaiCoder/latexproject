import { useEffect, useState } from "react";
import { Play } from "lucide-react";
import { compileLatexDocument, loadLatexCompilers } from "../backend/latexBackend";
import type { CompileLatexDocumentResult, LatexCompiler } from "../domain/latexCompiler";

const STARTER_DOCUMENT = "\\documentclass{article}\n\\begin{document}\nHello from LaTeX Workbench.\n\\end{document}\n";

export function App() {
  const [compilers, setCompilers] = useState<LatexCompiler[]>([]);
  const [compilerError, setCompilerError] = useState(false);
  const [compileResult, setCompileResult] = useState<CompileLatexDocumentResult | undefined>();
  const [compileError, setCompileError] = useState(false);
  const [isCompiling, setIsCompiling] = useState(false);
  const [source, setSource] = useState(STARTER_DOCUMENT);

  useEffect(() => {
    let isMounted = true;

    loadLatexCompilers()
      .then((loadedCompilers) => {
        if (isMounted) {
          setCompilers(loadedCompilers);
          setCompilerError(false);
        }
      })
      .catch(() => {
        if (isMounted) {
          setCompilers([]);
          setCompilerError(true);
        }
      });

    return () => {
      isMounted = false;
    };
  }, []);

  const defaultCompiler = compilers.find((compiler) => compiler.isDefault);
  const compilerSummary = compilers.map((compiler) => compiler.label).join(", ");
  const formatCompilerStatus = (compiler: LatexCompiler) => {
    if (compiler.status === "installed") {
      return "Installed";
    }

    if (compiler.statusReason === "notFound") {
      return "Missing (not found on PATH)";
    }

    if (compiler.statusReason === "failed") {
      return "Missing (version check failed)";
    }

    if (compiler.statusReason === "timeout") {
      return "Missing (version check timed out)";
    }

    return "Missing";
  };

  const handleCompile = async () => {
    if (!defaultCompiler || isCompiling) {
      return;
    }

    setIsCompiling(true);
    setCompileError(false);
    setCompileResult(undefined);

    try {
      const result = await compileLatexDocument({
        compilerId: defaultCompiler.id,
        source,
      });
      setCompileResult(result);
    } catch {
      setCompileError(true);
    } finally {
      setIsCompiling(false);
    }
  };

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand" aria-label="LaTeX Workbench">
          <span className="brand-icon">T</span>
          <span>LaTeX Workbench</span>
        </div>
        <button
          className="compile-button"
          type="button"
          disabled={!defaultCompiler || isCompiling}
          onClick={handleCompile}
        >
          <Play size={16} aria-hidden="true" /> {isCompiling ? "Compiling" : "Compile"}
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
          <div className="editor-surface">
            <label className="visually-hidden" htmlFor="latex-source">
              main.tex source
            </label>
            <textarea
              id="latex-source"
              className="source-editor"
              spellCheck={false}
              value={source}
              onChange={(event) => setSource(event.target.value)}
            />
            {compilerError ? (
              <p role="status">Unable to load LaTeX compilers.</p>
            ) : (
              <>
                <p>Default compiler: {defaultCompiler?.label ?? "Loading..."}</p>
                {compilerSummary ? <p>Available compilers: {compilerSummary}</p> : null}
                {compilers.length > 0 ? (
                  <ul className="compiler-status-list" aria-label="LaTeX compiler status">
                    {compilers.map((compiler) => (
                      <li key={compiler.id}>
                        {compiler.label}: {formatCompilerStatus(compiler)}
                      </li>
                    ))}
                  </ul>
                ) : null}
              </>
            )}
            {compileError ? <p role="status">Compile request failed.</p> : null}
            {compileResult?.success && compileResult.pdfPath ? (
              <p role="status">Compile succeeded: {compileResult.pdfPath}</p>
            ) : null}
            {compileResult?.success && compileResult.log ? (
              <section className="compile-log" aria-label="Compile log">
                <pre>{compileResult.log}</pre>
              </section>
            ) : null}
            {compileResult && !compileResult.success ? (
              <section className="compile-log" aria-label="Compile log" role="status">
                <p>Compile failed.</p>
                <pre>{compileResult.log}</pre>
              </section>
            ) : null}
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
