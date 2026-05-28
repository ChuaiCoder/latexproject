import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadLatexEngines } from "./latexBackend";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import("@tauri-apps/api/core");

describe("latex backend client", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("loads LaTeX engines through the Tauri command boundary", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { id: "miktex", label: "MiKTeX", isDefault: true },
      { id: "tectonic", label: "Tectonic", isDefault: false },
    ]);

    const engines = await loadLatexEngines();

    expect(invoke).toHaveBeenCalledWith("available_latex_engines");
    expect(engines).toEqual([
      { id: "miktex", label: "MiKTeX", isDefault: true },
      { id: "tectonic", label: "Tectonic", isDefault: false },
    ]);
  });
});

