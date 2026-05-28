import { describe, expect, it } from "vitest";
import { getPreferredEngine } from "./latexEngine";

describe("latex engine selection", () => {
  it("keeps a preferred engine when the backend reports it as available", () => {
    expect(getPreferredEngine("tectonic", ["miktex", "tectonic"])).toBe("tectonic");
  });

  it("falls back to the backend default engine for unsupported preferences", () => {
    expect(getPreferredEngine("unknown", ["tectonic", "miktex"])).toBe("tectonic");
  });
});
