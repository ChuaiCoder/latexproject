export type LatexEngineId = string;
export type LatexEngineStatus = "installed" | "missing";
export type LatexEngineStatusReason = "available" | "notFound" | "failed" | "timeout";

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
): LatexEngineId | undefined {
  const [fallbackEngine] = availableEngineIds;

  if (preference && availableEngineIds.includes(preference)) {
    return preference;
  }

  return fallbackEngine;
}
