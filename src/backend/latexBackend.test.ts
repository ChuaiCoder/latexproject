import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadLatexEngines, resetLatexEnginesCacheForTests } from "./latexBackend";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import("@tauri-apps/api/core");

describe("latex backend client", () => {
  beforeEach(() => {
    resetLatexEnginesCacheForTests();
    vi.mocked(invoke).mockReset();
  });

  it("loads LaTeX engines through the Tauri command boundary", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: "miktex", label: "MiKTeX", isDefault: true, status: "installed" },
      {
        id: "xelatex",
        label: "XeLaTeX",
        isDefault: false,
        status: "missing",
        statusReason: "notFound",
      },
    ]);

    const engines = await loadLatexEngines();

    expect(invoke).toHaveBeenCalledWith("available_latex_engines");
    expect(engines).toEqual([
      { id: "miktex", label: "MiKTeX", isDefault: true, status: "installed" },
      {
        id: "xelatex",
        label: "XeLaTeX",
        isDefault: false,
        status: "missing",
        statusReason: "notFound",
      },
    ]);
  });

  it("reuses an in-flight LaTeX engine request", async () => {
    const backendEngines = [
      { id: "miktex", label: "MiKTeX", isDefault: true, status: "installed" as const },
    ];
    vi.mocked(invoke).mockResolvedValue(backendEngines);

    const [firstResult, secondResult] = await Promise.all([
      loadLatexEngines(),
      loadLatexEngines(),
    ]);

    expect(invoke).toHaveBeenCalledTimes(1);
    expect(firstResult).toEqual(backendEngines);
    expect(secondResult).toEqual(backendEngines);
  });

  it("clears the cached request after a backend failure", async () => {
    const backendEngines = [
      { id: "miktex", label: "MiKTeX", isDefault: true, status: "installed" as const },
    ];
    vi.mocked(invoke)
      .mockRejectedValueOnce(new Error("backend unavailable"))
      .mockResolvedValueOnce(backendEngines);

    await expect(loadLatexEngines()).rejects.toThrow("backend unavailable");
    await expect(loadLatexEngines()).resolves.toEqual(backendEngines);

    expect(invoke).toHaveBeenCalledTimes(2);
  });
});
