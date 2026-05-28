import { invoke } from "@tauri-apps/api/core";
import type { LatexEngine } from "../domain/latexEngine";

let latexEnginesRequest: Promise<LatexEngine[]> | undefined;

export async function loadLatexEngines(): Promise<LatexEngine[]> {
  latexEnginesRequest ??= invoke<LatexEngine[]>("available_latex_engines").catch((error) => {
    latexEnginesRequest = undefined;
    throw error;
  });

  return latexEnginesRequest;
}

export function resetLatexEnginesCacheForTests(): void {
  latexEnginesRequest = undefined;
}
