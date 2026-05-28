import { describe, expect, it } from "vitest";
import { getPreferredCompiler } from "./latexCompiler";

describe("latex compiler selection", () => {
  it("keeps a preferred compiler when the backend reports it as available", () => {
    expect(getPreferredCompiler("xelatex", ["pdflatex", "xelatex"])).toBe("xelatex");
  });

  it("keeps a preferred compiler whose id is supplied by the backend", () => {
    expect(getPreferredCompiler("typst", ["pdflatex", "typst"])).toBe("typst");
  });

  it("falls back to the backend default compiler for unsupported preferences", () => {
    expect(getPreferredCompiler("unknown", ["pdflatex", "xelatex"])).toBe("pdflatex");
  });

  it("returns no preferred compiler when the backend reports no compilers", () => {
    expect(getPreferredCompiler("pdflatex", [])).toBeUndefined();
  });
});
