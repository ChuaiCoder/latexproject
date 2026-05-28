import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { compileLatexDocument, loadLatexCompilers } from "../backend/latexBackend";

vi.mock("../backend/latexBackend", () => ({
  compileLatexDocument: vi.fn(),
  loadLatexCompilers: vi.fn(),
}));

describe("App", () => {
  beforeEach(() => {
    vi.mocked(compileLatexDocument).mockReset();
    vi.mocked(loadLatexCompilers).mockReset();
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
    expect(await screen.findByText("Default compiler: pdfLaTeX")).toBeInTheDocument();
    expect(screen.getByText("Available compilers: pdfLaTeX, XeLaTeX")).toBeInTheDocument();
    expect(screen.getByText("pdfLaTeX: Installed")).toBeInTheDocument();
    expect(screen.getByText("XeLaTeX: Missing (not found on PATH)")).toBeInTheDocument();
  });

  it("shows a backend error when LaTeX compilers cannot be loaded", async () => {
    vi.mocked(loadLatexCompilers).mockRejectedValue(new Error("backend unavailable"));

    render(<App />);

    expect(await screen.findByText("Unable to load LaTeX compilers.")).toBeInTheDocument();
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
  });

  it("compiles the starter document with the default compiler", async () => {
    vi.mocked(loadLatexCompilers).mockResolvedValue([
      { id: "pdflatex", label: "pdfLaTeX", isDefault: true, status: "installed" },
    ]);
    vi.mocked(compileLatexDocument).mockResolvedValue({
      success: true,
      log: "compiled",
      pdfPath: "C:\\tmp\\main.pdf",
    });

    render(<App />);

    await screen.findByText("Default compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    await waitFor(() => {
      expect(compileLatexDocument).toHaveBeenCalledWith({
        compilerId: "pdflatex",
        source: expect.stringContaining("\\documentclass{article}"),
      });
    });
    expect(await screen.findByText("Compile succeeded: C:\\tmp\\main.pdf")).toBeInTheDocument();
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

    await screen.findByText("Default compiler: pdfLaTeX");
    fireEvent.click(screen.getByRole("button", { name: /compile/i }));

    expect(await screen.findByText("Compile failed.")).toBeInTheDocument();
    expect(screen.getByText("! Undefined control sequence.")).toBeInTheDocument();
  });
});
