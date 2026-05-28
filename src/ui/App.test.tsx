import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("shows the workbench shell with the default LaTeX engine", () => {
    render(<App />);

    expect(screen.getByLabelText("LaTeX Workbench")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /compile/i })).toBeInTheDocument();
    expect(screen.getByText("Default engine: miktex")).toBeInTheDocument();
  });
});

