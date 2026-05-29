import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const cargoExecutableName = process.platform === "win32" ? "cargo.exe" : "cargo";

function getPathKey(env) {
  return Object.keys(env).find((key) => key.toLowerCase() === "path") ?? "PATH";
}

function getHomeDir(env) {
  if (process.platform === "win32") {
    return env.USERPROFILE;
  }

  return env.HOME;
}

export function withCargoBinPath(env = process.env, exists = fs.existsSync) {
  const nextEnv = { ...env };
  const homeDir = getHomeDir(nextEnv) ?? os.homedir();
  const cargoBin = path.join(homeDir, ".cargo", "bin");
  const cargoExecutable = path.join(cargoBin, cargoExecutableName);

  if (!exists(cargoExecutable)) {
    return nextEnv;
  }

  const pathKey = getPathKey(nextEnv);
  const currentPath = nextEnv[pathKey] ?? "";
  const pathEntries = currentPath.split(path.delimiter).filter(Boolean);
  const normalizedCargoBin = normalizePathForComparison(cargoBin);
  const alreadyInPath = pathEntries.some((entry) => normalizePathForComparison(entry) === normalizedCargoBin);

  if (!alreadyInPath) {
    nextEnv[pathKey] = [cargoBin, currentPath].filter(Boolean).join(path.delimiter);
  }

  return nextEnv;
}

function normalizePathForComparison(candidate) {
  const normalized = path.resolve(candidate);

  if (process.platform === "win32") {
    return normalized.toLowerCase();
  }

  return normalized;
}

function runTauri() {
  const tauriCommand = getTauriCommand(process.cwd(), process.platform);
  const result = spawnSync(tauriCommand.command, process.argv.slice(2), {
    env: withCargoBinPath(),
    shell: tauriCommand.shell,
    stdio: "inherit",
  });

  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }

  process.exit(result.status ?? 1);
}

export function getTauriCommand(cwd = process.cwd(), platform = process.platform) {
  if (platform === "win32") {
    return {
      command: path.join(cwd, "node_modules", ".bin", "tauri.cmd"),
      shell: true,
    };
  }

  return {
    command: path.join(cwd, "node_modules", ".bin", "tauri"),
    shell: false,
  };
}

export function isMainModule(moduleUrl, executedPath) {
  return path.resolve(fileURLToPath(moduleUrl)) === path.resolve(executedPath);
}

if (isMainModule(import.meta.url, process.argv[1])) {
  runTauri();
}
