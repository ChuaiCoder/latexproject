import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import {
  compileLatexDocument,
  installLatexToolchain,
  loadLatexCompilers,
  loadLatexDependencyState,
} from "../backend/latexBackend";
import { convertFileSrc, isTauri } from "@tauri-apps/api/core";

vi.mock("../backend/latexBackend", () => ({
  compileLatexDocument: vi.fn(),
  installLatexToolchain: vi.fn(),
  loadLatexCompilers: vi.fn(),
  loadLatexDependencyState: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://${path}`),
  isTauri: vi.fn(() => true),
}));

describe("App", () => {
  beforeEach(() => {
    vi.mocked(compileLatexDocument).mockReset();
    vi.mocked(installLatexToolchain).mockReset();
    vi.mocked(loadLatexCompilers).mockReset();
    vi.mocked(loadLatexDependencyState).mockReset();
    vi.mocked(convertFileSrc).mockClear();
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(loadLatexDependencyState).mockResolvedValue({
      toolchainsDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains",
      packagesDir: "C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\packages",
      managedToolchains: [],
    });
  });

  it("shows the workbench shell with the default LaTeX compiler from the backend", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
      {
        id: "xelatex",
        label: "XeLaTeX",
        isDefault: false,
        status: "missing",
        statusReason: "notFound",
      },
    ]);

    render(<App />);

    expect(screen.getByLabelText("LaTeX Workbench")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /compile/i })).toBeInTheDocument();
    expect(await screen.findByText("Selected compiler: pdfLaTeX")).toBeInTheDocument();
    expect(screen.getByText("Available compilers: pdfLaTeX, XeLaTeX")).toBeInTheDocument();
    expect(screen.getByText("pdfLaTeX: Installed")).toBeInTheDocument();
    expect(screen.getByText("XeLaTeX: Missing (not found on PATH)")).toBeInTheDocument();
  });

  it("shows where app-managed LaTeX dependencies are stored", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(loadLatexDependencyState).mockResolvedValue({
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
    });

    render(<App />);

    expect(await screen.findByText("Managed dependencies")).toBeInTheDocument();
    expect(screen.getByText("Toolchains: C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\toolchains")).toBeInTheDocument();
    expect(screen.getByText("Packages: C:\\Users\\Dev\\AppData\\Local\\LatexWorkbench\\packages")).toBeInTheDocument();
    expect(screen.getByText("Tectonic: Missing")).toBeInTheDocument();
  });

  it("installs Tectonic and refreshes compiler state", async () => {
    vi.mocked(loadLatexCompilers)
      .mockResolvedValueOnce([
        {
          id: "tectonic",
          label: "Tectonic",
          isDefault: false,
          status: "missing",
          statusReason: "notFound",
        },
      ])
      .mockResolvedValueOnce([
        { id: "tectonic", label: "Tectonic", isDefault: true, status: "installed" },
      ]);
    vi.mocked(loadLatexDependencyState).mockResolvedValue({
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
    });
    vi.mocked(installLatexToolchain).mockResolvedValue({
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
    });

    render(<App />);

    await screen.findByText("Selected compiler: No installed compiler");
    fireEvent.click(screen.getByRole("button", { name: "Install Tectonic" }));

    await waitFor(() => {
      expect(installLatexToolchain).toHaveBeenCalledWith({ toolchainId: "tectonic" });
    });
    expect(await screen.findByText("Tectonic installed.")).toBeInTheDocument();
    expect(await screen.findByText("Selected compiler: Tectonic")).toBeInTheDocument();
  });

  it("shows a backend error when LaTeX compilers cannot be loaded", async () => {
    vi.mocked(loadLatexCompilers).mockRejectedValue(new Error("backend unavailable"));

    render(<App />);

    expect(await screen.findByText("Unable to load LaTeX compilers.")).toBeInTheDocument();
  });

  it("explains that compiling requires the Tauri desktop client", () => {
    vi.mocked(isTauri).mockReturnValue(false);

    render(<App />);

    expect(screen.getByText("Desktop client required for LaTeX compilation.")).toBeInTheDocument();
    expect(screen.getByText("Run npm run tauri dev to use the Rust compiler backend.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /compile/i })).toBeDisabled();
    expect(loadLatexCompilers).not.toHaveBeenCalled();
    expect(loadLatexDependencyState).not.toHaveBeenCalled();
  });

  it("shows when a LaTeX compiler detection times out", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      {
        id: "pdflatex",
        label: "pdfLaTeX",
        isDefault: true,
        status: "missing",
        statusReason: "timeout",
      },
    ]);

    render(<App />);

    expect(await screen.findByText("pdfLaTeX: Missing (version check timed out)")).toBeInTheDocument();
    expect(screen.getByText("Selected compiler: No installed compiler")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /compile/i })).toBeDisabled();
  });

  it("falls back to the first installed compiler when the backend default is missing", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      {
        id: "pdflatex",
        label: "pdfLaTeX",
        isDefault: true,
        status: "missing",
        statusReason: "notFound",
      },
      { id: "xelatex", label: "XeLaTeX", isDefault: false, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "compiled with xelatex",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    expect(await screen.findByText("Selected compiler: XeLaTeX")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    await waitFor(() => {
      expect(compileLatexDocument).toHaveBeenCalledWith({
        compilerId: "xelatex",
        source: expect.stringContaining("\\documentclass{article}"),
      });
    });
  });

  it("compiles the starter document with the selected compiler", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "compiled",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    await waitFor(() => {
      expect(compileLatexDocument).toHaveBeenCalledWith({
        compilerId: "pdflatex",
        source: expect.stringContaining("\\documentclass{article}"),
      });
    });
    expect(await screen.findByText("Compile succeeded: C:\\tmp\\main.pdf")).toBeInTheDocument();
  });

  it("compiles the current editor contents", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "compiled edited document",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.change(screen.getByLabelText("main.tex source"), {
      target: {
        value: "\\documentclass{article}\n\\begin{document}\nEdited source\n\\end{document}\n",
      },
    });
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    await waitFor(() => {
      expect(compileLatexDocument).toHaveBeenCalledWith({
        compilerId: "pdflatex",
        source: expect.stringContaining("Edited source"),
      });
    });
  });

  it("shows the compiler log after successful compiles", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "Output written on main.pdf",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    const logPanel = await screen.findByRole("region", { name: "Compile log" });
    expect(within(logPanel).getByText("Output written on main.pdf")).toBeInTheDocument();
  });

  it("shows compile failures from the backend", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: false,
      log: "! Undefined control sequence.",
    });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    expect(await screen.findByText("Compile failed.")).toBeInTheDocument();
    expect(screen.getByText("! Undefined control sequence.")).toBeInTheDocument();
  });

  it("shows compile request errors in the compile log", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockRejectedValue(new Error("spawn pdflatex ENOENT"));

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    expect(await screen.findByText("Compile failed.")).toBeInTheDocument();
    expect(screen.getByText("spawn pdflatex ENOENT")).toBeInTheDocument();
  });

  it("previews the compiled PDF after a successful compile", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "compiled",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    const preview = await screen.findByTitle("Compiled PDF preview");
    expect(convertFileSrc).toHaveBeenCalledWith("C:\\tmp\\main.pdf");
    expect(preview).toHaveAttribute("src", "asset://C:\\tmp\\main.pdf");
  });

  it("clears the previous PDF preview when the next compile fails", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument)
      .mockResolvedValueOnce({
        success: true,
        log: "compiled",
        pdfPath: "C:\\tmp\\main.pdf",
      })
      .mockResolvedValueOnce({
        success: false,
        log: "! Undefined control sequence.",
      });

    render(<App />);

    await screen.findByText("Selected compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));
    expect(await screen.findByTitle("Compiled PDF preview")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    await screen.findByText("Compile failed.");
    expect(screen.queryByTitle("Compiled PDF preview")).not.toBeInTheDocument();
    expect(screen.getByText("No PDF preview available.")).toBeInTheDocument();
  });
});
