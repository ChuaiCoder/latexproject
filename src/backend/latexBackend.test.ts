import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  compileLatexDocument,
  installLatexToolchain,
  loadLatexDependencyState,
  loadLatexCompilers,
  resetLatexCompilersCacheForTests,
} from "./latexBackend";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import("@tauri-apps/api/core");

describe("latex backend client", () => {
  beforeEach(() => {
    resetLatexCompilersCacheForTests();
    vi.mocked(invoke).mockReset();
  });

  it("loads LaTeX compilers through the Tauri command boundary", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
      {
        id: "xelatex",
        label: "XeLaTeX",
        isDefault: false,
        status: "missing",
        statusReason: "notFound",
      },
    ]);

    const compilers = await loadLatexCompilers();

    expect(invoke).toHaveBeenCalledWith("available_latex_compilers");
    expect(compilers).toEqual([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
      {
        id: "xelatex",
        label: "XeLaTeX",
        isDefault: false,
        status: "missing",
        statusReason: "notFound",
      },
    ]);
  });

  it("reuses an in-flight LaTeX compiler request", async () => {
    const backendCompilers = [
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" as const },
    ];
    vi.mocked(invoke).mockResolvedValue(backendCompilers);

    const [firstResult, secondResult] = await Promise.all([
      loadLatexCompilers(),
      loadLatexCompilers(),
    ]);

    expect(invoke).toHaveBeenCalledTimes(1);
    expect(firstResult).toEqual(backendCompilers);
    expect(secondResult).toEqual(backendCompilers);
  });

  it("clears the cached compiler request after a backend failure", async () => {
    const backendCompilers = [
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" as const },
    ];
    vi.mocked(invoke)
      .mockRejectedValueOnce(new Error("backend unavailable"))
      .mockResolvedValueOnce(backendCompilers);

    await expect(loadLatexCompilers()).rejects.toThrow("backend unavailable");
    await expect(loadLatexCompilers()).resolves.toEqual(backendCompilers);

    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("sends a LaTeX document compile request through the Tauri command boundary", async () => {
    vi.mocked(invoke).mockResolvedValue({
      success: true,
      log: "compiled",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    const result = await compileLatexDocument({
      compilerId: "pdflatex",
      source: "\\documentclass{article}\\begin{document}Hi\\end{document}",
    });

    expect(invoke).toHaveBeenCalledWith("compile_latex_document", {
      request: {
        compilerId: "pdflatex",
        source: "\\documentclass{article}\\begin{document}Hi\\end{document}",
      },
    });
    expect(result).toEqual({
      success: true,
      log: "compiled",
      pdfPath: "C:\\tmp\\main.pdf",
    });
  });

  it("loads the managed LaTeX dependency state through the Tauri command boundary", async () => {
    const dependencyState = {
      toolchainsDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains",
      packagesDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\packages",
      managedToolchains: [
        {
          id: "tectonic",
          label: "Tectonic",
          installDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains\\tectonic",
          executablePath: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains\\tectonic\\tectonic.exe",
          compilerIds: ["tectonic"],
          status: "missing",
        },
      ],
    };
    vi.mocked(invoke).mockResolvedValue(dependencyState);

    const result = await loadLatexDependencyState();

    expect(invoke).toHaveBeenCalledWith("latex_dependency_state");
    expect(result).toEqual(dependencyState);
  });

  it("installs a managed LaTeX toolchain through the Tauri command boundary", async () => {
    const installResult = {
      success: true,
      log: "Tectonic installed.",
      dependencyState: {
        toolchainsDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains",
        packagesDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\packages",
        managedToolchains: [
          {
            id: "tectonic",
            label: "Tectonic",
            installDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains\\tectonic",
            executablePath: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains\\tectonic\\tectonic.exe",
            compilerIds: ["tectonic"],
            status: "installed",
          },
        ],
      },
    };
    vi.mocked(invoke).mockResolvedValue(installResult);

    const result = await installLatexToolchain({ toolchainId: "tectonic" });

    expect(invoke).toHaveBeenCalledWith("install_latex_toolchain", {
      request: { toolchainId: "tectonic" },
    });
    expect(result).toEqual(installResult);
  });
});
