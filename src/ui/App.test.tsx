import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { loadLatexEngines } from "../backend/latexBackend";

vi.mock("../backend/latexBackend", () => ({
  loadLatexEngines: vi.fn(),
}));

describe("App", () => {
  beforeEach(() => {
    vi.mocked(loadLatexEngines).mockReset();
  });

  it("shows the workbench shell with the default LaTeX engine from the backend", async () => {
    vi.mocked(loadLatexEngines).mockResolvedValue([
      { id: "miktex", label: "MiKTeX", isDefault: true, status: "installed" },
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
    expect(await screen.findByText("Default engine: MiKTeX")).toBeInTheDocument();
    expect(screen.getByText("Available engines: MiKTeX, XeLaTeX")).toBeInTheDocument();
    expect(screen.getByText("MiKTeX: Installed")).toBeInTheDocument();
    expect(screen.getByText("XeLaTeX: Missing (not found on PATH)")).toBeInTheDocument();
  });

  it("shows a backend error when LaTeX engines cannot be loaded", async () => {
    vi.mocked(loadLatexEngines).mockRejectedValue(new Error("backend unavailable"));

    render(<App />);

    expect(await screen.findByText("Unable to load LaTeX engines.")).toBeInTheDocument();
  });
});
