import path from "node:path";
import { pathToFileURL } from "node:url";
import { describe, expect, it } from "vitest";
import { getTauriCommand, isMainModule, withCargoBinPath } from "./tauri-env.mjs";

const cargoExecutableName = process.platform === "win32" ? "cargo.exe" : "cargo";

describe("withCargoBinPath", () => {
  it("prepends the default rustup cargo bin when it exists but is missing from PATH", () => {
    const userProfile = process.platform === "win32" ? "C:\\Users\\Dev" : "/home/dev";
    const cargoBin = path.join(userProfile, ".cargo", "bin");
    const cargoExecutable = path.join(cargoBin, cargoExecutableName);
    const existingPath = [path.join(userProfile, "AppData", "Local", "Programs"), "C:\\Windows\\System32"].join(
      path.delimiter,
    );

    const env = withCargoBinPath(
      {
        USERPROFILE: userProfile,
        Path: existingPath,
      },
      (candidate) => candidate === cargoExecutable,
    );

    expect(env.Path.split(path.delimiter)[0]).toBe(cargoBin);
    expect(env.Path).toContain(existingPath);
  });

  it("does not duplicate the cargo bin when PATH already contains it", () => {
    const userProfile = process.platform === "win32" ? "C:\\Users\\Dev" : "/home/dev";
    const cargoBin = path.join(userProfile, ".cargo", "bin");
    const existingPath = [cargoBin, "C:\\Windows\\System32"].join(path.delimiter);

    const env = withCargoBinPath(
      {
        USERPROFILE: userProfile,
        Path: existingPath,
      },
      () => true,
    );

    expect(env.Path).toBe(existingPath);
  });

  it("leaves PATH unchanged when cargo is not installed in the default rustup location", () => {
    const existingPath = ["C:\\Windows\\System32", "C:\\Tools"].join(path.delimiter);

    const env = withCargoBinPath(
      {
        USERPROFILE: process.platform === "win32" ? "C:\\Users\\Dev" : "/home/dev",
        Path: existingPath,
      },
      () => false,
    );

    expect(env.Path).toBe(existingPath);
  });

  it("detects when the wrapper is executed as the main script", () => {
    const scriptPath = path.resolve("scripts", "tauri-env.mjs");

    expect(isMainModule(pathToFileURL(scriptPath).href, scriptPath)).toBe(true);
    expect(isMainModule(pathToFileURL(scriptPath).href, path.resolve("scripts", "other.mjs"))).toBe(false);
  });

  it("uses the local Tauri CLI shim", () => {
    const command = getTauriCommand("E:\\project", "win32");

    expect(command.command).toBe(path.join("E:\\project", "node_modules", ".bin", "tauri.cmd"));
    expect(command.shell).toBe(true);
  });
});
