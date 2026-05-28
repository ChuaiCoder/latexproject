export type LatexEngineId = "miktex" | "tectonic";

export interface LatexEngine {
  id: LatexEngineId;
  label: string;
  isDefault: boolean;
}

export const DEFAULT_ENGINE_ID: LatexEngineId = "miktex";

const SUPPORTED_ENGINE_IDS = new Set<LatexEngineId>(["miktex", "tectonic"]);

export function getPreferredEngine(preference: string | null | undefined): LatexEngineId {
  if (preference && SUPPORTED_ENGINE_IDS.has(preference as LatexEngineId)) {
    return preference as LatexEngineId;
  }

  return DEFAULT_ENGINE_ID;
}
