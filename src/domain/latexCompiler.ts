export type LatexCompilerId = string;
export type LatexCompilerStatus = "installed" | "missing";
export type LatexCompilerStatusReason = "available" | "notFound" | "failed" | "timeout";

export interface LatexCompiler {
  id: LatexCompilerId;
  label: string;
  isDefault: boolean;
  status: LatexCompilerStatus;
  statusReason?: LatexCompilerStatusReason;
}

export interface CompileLatexDocumentRequest {
  compilerId: LatexCompilerId;
  source: string;
}

export interface CompileLatexDocumentResult {
  success: boolean;
  log: string;
  pdfPath?: string;
}

export function getPreferredCompiler(
  preference: string | null | undefined,
  availableCompilerIds: LatexCompilerId[],
): LatexCompilerId | undefined {
  const [fallbackCompiler] = availableCompilerIds;

  if (preference && availableCompilerIds.includes(preference)) {
    return preference;
  }

  return fallbackCompiler;
}
