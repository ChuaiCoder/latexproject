# LaTeX Workbench

A desktop LaTeX workbench built with Tauri, React, TypeScript, and Rust.

## Development

```powershell
npm install
npm test
npm run build
npm run tauri dev
```

## Architecture

- `src/` contains the React front end.
- `src/domain/` contains UI-independent TypeScript domain logic.
- `src-tauri/` contains the Rust desktop backend.
- `src-tauri/src/latex/` contains LaTeX engine abstractions.

