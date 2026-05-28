import { invoke } from "@tauri-apps/api/core";
import type {
  CompileLatexDocumentRequest,
  CompileLatexDocumentResult,
  LatexCompiler,
} from "../domain/latexCompiler";

let latexCompilersRequest: Promise<LatexCompiler[]> | undefined;

export async function loadLatexCompilers(): Promise<LatexCompiler[]> {
  latexCompilersRequest ??= invoke<LatexCompiler[]>("available_latex_compilers").catch((error) => {
    latexCompilersRequest = undefined;
    throw error;
  });

  return latexCompilersRequest;
}

export async function compileLatexDocument(
  request: CompileLatexDocumentRequest,
): Promise<CompileLatexDocumentResult> {
  return invoke<CompileLatexDocumentResult>("compile_latex_document", { request });
}

export function resetLatexCompilersCacheForTests(): void {
  latexCompilersRequest = undefined;
}
