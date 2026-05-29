import { invoke } from "@tauri-apps/api/core";
import type {
  CompileLatexDocumentRequest,
  CompileLatexDocumentResult,
  InstallLatexToolchainRequest,
  InstallLatexToolchainResult,
  LatexDependencyState,
  LatexCompiler,
} from "../domain/latexCompiler";

let latexCompilersRequest: Promise<LatexCompiler[]> | undefined;
let latexDependencyStateRequest: Promise<LatexDependencyState> | undefined;

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

export async function loadLatexDependencyState(): Promise<LatexDependencyState> {
  latexDependencyStateRequest ??= invoke<LatexDependencyState>("latex_dependency_state").catch((error) => {
    latexDependencyStateRequest = undefined;
    throw error;
  });

  return latexDependencyStateRequest;
}

export async function installLatexToolchain(
  request: InstallLatexToolchainRequest,
): Promise<InstallLatexToolchainResult> {
  const result = await invoke<InstallLatexToolchainResult>("install_latex_toolchain", { request });
  latexCompilersRequest = undefined;
  latexDependencyStateRequest = Promise.resolve(result.dependencyState);
  return result;
}

export function resetLatexCompilersCacheForTests(): void {
  latexCompilersRequest = undefined;
  latexDependencyStateRequest = undefined;
}
