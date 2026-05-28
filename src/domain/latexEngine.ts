export type LatexEngineId = "miktex" | "tectonic";
export type LatexEngineStatus = "installed" | "missing";
export type LatexEngineStatusReason = "available" | "notFound" | "failed";

export interface LatexEngine {
  id: LatexEngineId;
  label: string;
  isDefault: boolean;
  status: LatexEngineStatus;
  statusReason?: LatexEngineStatusReason;
}

export function getPreferredEngine(
  preference: string | null | undefined,
  availableEngineIds: LatexEngineId[],
): LatexEngineId {
  const [fallbackEngine] = availableEngineIds;

  if (preference && availableEngineIds.includes(preference as LatexEngineId)) {
    return preference as LatexEngineId;
  }

  return fallbackEngine;
}
