import { useEffect, useState } from "react";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { Download, Play } from "lucide-react";
import {
  compileLatexDocument,
  installLatexToolchain,
  loadLatexCompilers,
  loadLatexDependencyState,
} from "../backend/latexBackend";
import type { CompileLatexDocumentResult, LatexCompiler, LatexDependencyState } from "../domain/latexCompiler";

const STARTER_DOCUMENT = "\\documentclass{article}\n\\begin{document}\nHello from LaTeX Workbench.\n\\end{document}\n";

export function App() {
  const [compilers, setCompilers] = useState<LatexCompiler[]>([]);
  const [compilerError, setCompilerError] = useState(false);
  const [dependencyState, setDependencyState] = useState<LatexDependencyState | undefined>();
  const [dependencyError, setDependencyError] = useState(false);
  const [compileResult, setCompileResult] = useState<CompileLatexDocumentResult | undefined>();
  const [isCompiling, setIsCompiling] = useState(false);
  const [isInstallingToolchain, setIsInstallingToolchain] = useState(false);
  const [installLog, setInstallLog] = useState<string | undefined>();
  const [source, setSource] = useState(STARTER_DOCUMENT);
  const isDesktopClient = isTauri();

  useEffect(() => {
    if (!isDesktopClient) {
      return;
    }

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

    loadLatexDependencyState()
      .then((loadedDependencyState) => {
        if (isMounted) {
          setDependencyState(loadedDependencyState);
          setDependencyError(false);
        }
      })
      .catch(() => {
        if (isMounted) {
          setDependencyState(undefined);
          setDependencyError(true);
        }
      });

    return () => {
      isMounted = false;
    };
  }, [isDesktopClient]);

  const selectedCompiler =
    compilers.find((compiler) => compiler.isDefault && compiler.status === "installed") ??
    compilers.find((compiler) => compiler.status === "installed");
  const installableTectonic = dependencyState?.managedToolchains.find(
    (toolchain) => toolchain.id === "tectonic" && toolchain.status === "missing",
  );
  const previewSource =
    compileResult?.success && compileResult.pdfPath ? convertFileSrc(compileResult.pdfPath) : undefined;
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
    if (!isDesktopClient || !selectedCompiler || isCompiling) {
      return;
    }

    setIsCompiling(true);
    setCompileResult(undefined);

    try {
      const result = await compileLatexDocument({
        compilerId: selectedCompiler.id,
        source,
      });
      setCompileResult(result);
    } catch (error) {
      setCompileResult({
        success: false,
        log: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setIsCompiling(false);
    }
  };

  const refreshLatexState = async () => {
    const [loadedCompilers, loadedDependencyState] = await Promise.all([
      loadLatexCompilers(),
      loadLatexDependencyState(),
    ]);
    setCompilers(loadedCompilers);
    setCompilerError(false);
    setDependencyState(loadedDependencyState);
    setDependencyError(false);
  };

  const handleInstallTectonic = async () => {
    if (!isDesktopClient || isInstallingToolchain) {
      return;
    }

    setIsInstallingToolchain(true);
    setInstallLog(undefined);

    try {
      const result = await installLatexToolchain({ toolchainId: "tectonic" });
      setDependencyState(result.dependencyState);
      setInstallLog(result.log);
      await refreshLatexState();
    } catch (error) {
      setInstallLog(error instanceof Error ? error.message : String(error));
    } finally {
      setIsInstallingToolchain(false);
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
          disabled={!isDesktopClient || !selectedCompiler || isCompiling}
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
            {!isDesktopClient ? (
              <section className="compile-log" aria-label="Desktop client notice" role="status">
                <p>Desktop client required for LaTeX compilation.</p>
                <pre>Run npm run tauri dev to use the Rust compiler backend.</pre>
              </section>
            ) : compilerError ? (
              <p role="status">Unable to load LaTeX compilers.</p>
            ) : (
              <>
                <p>Selected compiler: {selectedCompiler?.label ?? "No installed compiler"}</p>
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
                {dependencyState ? (
                  <section className="dependency-status" aria-label="Managed LaTeX dependencies">
                    <h2>Managed dependencies</h2>
                    <p>Toolchains: {dependencyState.toolchainsDir}</p>
                    <p>Packages: {dependencyState.packagesDir}</p>
                    {dependencyState.managedToolchains.length > 0 ? (
                      <ul className="compiler-status-list">
                        {dependencyState.managedToolchains.map((toolchain) => (
                          <li key={toolchain.id}>
                            {toolchain.label}: {toolchain.status === "installed" ? "Installed" : "Missing"}
                          </li>
                        ))}
                      </ul>
                    ) : null}
                    {installableTectonic ? (
                      <button
                        className="secondary-button"
                        type="button"
                        disabled={isInstallingToolchain}
                        onClick={handleInstallTectonic}
                      >
                        <Download size={16} aria-hidden="true" />
                        {isInstallingToolchain ? "Installing Tectonic" : "Install Tectonic"}
                      </button>
                    ) : null}
                  </section>
                ) : null}
                {dependencyError ? <p role="status">Unable to load managed LaTeX dependency state.</p> : null}
              </>
            )}
            {installLog ? (
              <section className="compile-log" aria-label="Install log" role="status">
                <pre>{installLog}</pre>
              </section>
            ) : null}
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
          {previewSource ? (
            <iframe
              className="pdf-preview"
              src={previewSource}
              title="Compiled PDF preview"
            />
          ) : (
            <div className="preview-placeholder">
              <p className="muted">No PDF preview available.</p>
            </div>
          )}
        </section>
      </section>
    </main>
  );
}
