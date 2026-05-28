import { describe, expect, it } from "vitest";
import { DEFAULT_ENGINE_ID, getPreferredEngine } from "./latexEngine";

describe("latex engine selection", () => {
  it("uses MiKTeX as the default engine for compatibility", () => {
    expect(DEFAULT_ENGINE_ID).toBe("miktex");
  });

  it("keeps an explicit supported engine preference", () => {
    expect(getPreferredEngine("tectonic")).toBe("tectonic");
  });

  it("falls back to the default engine for unsupported preferences", () => {
    expect(getPreferredEngine("unknown")).toBe(DEFAULT_ENGINE_ID);
  });
});

