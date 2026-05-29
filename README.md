# LaTeX Workbench

A desktop LaTeX workbench built with Tauri, React, TypeScript, and Rust.

## Development

```powershell
npm install
npm test
npm run build
npm run tauri dev
```

The `npm run tauri ...` script automatically adds the default rustup cargo directory
to the child process PATH when it exists. This keeps a stale terminal session from
failing with `cargo metadata: program not found` after Rust has been installed.

## Runtime Dependencies

End users should not need Node, npm, Rust, Cargo, or the Tauri CLI. Those are
development and build-time dependencies only.

LaTeX runtime dependencies are managed under the app data directory:

```text
<app-data>/dependencies/toolchains
<app-data>/dependencies/packages
```

The backend reports this state through `latex_dependency_state`. Managed
toolchains are detected from that directory first, then the app falls back to
LaTeX tools available on the system PATH.

## Architecture

- `src/` contains the React front end.
- `src/domain/` contains UI-independent TypeScript domain logic.
- `src-tauri/` contains the Rust desktop backend.
- `src-tauri/src/latex/` contains LaTeX engine abstractions.
