import { invoke } from "@tauri-apps/api/core";
import type { LatexEngine } from "../domain/latexEngine";

export async function loadLatexEngines(): Promise<LatexEngine[]> {
  return invoke<LatexEngine[]>("available_latex_engines");
}

