import * as vscode from "vscode";
import * as wasm from "rust-markdown-notebook";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { writeFile } from "node:fs/promises";
import { mkdir } from "node:fs/promises";
import { execFile as streamExecFile } from "node:child_process";
import { promisify } from "node:util";
import { rm } from "node:fs/promises";

const execFile = promisify(streamExecFile);

export class Controller {
  readonly controllerId = "rust-markdown-notebook-controller";
  readonly notebookType = "rust-markdown-notebook";
  readonly label = "Rust Markdown Notebook";
  readonly supportedLanguages = ["rust"];

  private readonly _controller: vscode.NotebookController;
  private _executionOrder = 0;

  constructor() {
    this._controller = vscode.notebooks.createNotebookController(
      this.controllerId,
      this.notebookType,
      this.label
    );

    this._controller.supportedLanguages = this.supportedLanguages;
    this._controller.supportsExecutionOrder = true;
    this._controller.executeHandler = this._execute.bind(this);
  }

  dispose() {
    this._controller.dispose();
  }

  private _execute(
    cells: vscode.NotebookCell[],
    _notebook: vscode.NotebookDocument,
    _controller: vscode.NotebookController
  ): void {
    for (let cell of cells) {
      this._doExecution(cell);
    }
  }

  private async _doExecution(cell: vscode.NotebookCell): Promise<void> {
    const execution = this._controller.createNotebookCellExecution(cell);
    try {
      execution.executionOrder = ++this._executionOrder;
      execution.start(Date.now()); // Keep track of elapsed time to execute cell.

      const source = wasm.to_executable_source(getNotebookSource(cell));
      const [output, error] = await executeSource(source, generateCargofile());

      execution.replaceOutput([
        new vscode.NotebookCellOutput([
          vscode.NotebookCellOutputItem.stdout(output),
          vscode.NotebookCellOutputItem.stderr(error),
        ]),
      ]);
    } catch (e: unknown) {
      execution.replaceOutput([
        new vscode.NotebookCellOutput([
          vscode.NotebookCellOutputItem.error(e as Error),
        ]),
      ]);
    } finally {
      execution.end(true, Date.now());
    }
  }
}

function getNotebookSource(cell: vscode.NotebookCell) {
  const previousCells = cell.notebook.getCells(
    new vscode.NotebookRange(0, cell.index)
  );
  const sources = [
    ...previousCells
      .filter((cell) => cell.document.languageId === "rust")
      .map((cell) => cell.document.getText()),
    cell.document.getText(),
  ];

  return sources.join("\n");
}

function generateCargofile() {
  return '[package]\nname = "tmp"\nversion = "0.1.0"';
}

async function executeSource(
  source: string,
  cargofile: string
): Promise<[string, string]> {
  return withScratchDir(source, cargofile, async (dir) => {
    const srcDir = path.join(dir, "src");
    await mkdir(srcDir);
    await Promise.all([
      writeFile(path.join(dir, "Cargo.toml"), cargofile),
      writeFile(path.join(srcDir, "main.rs"), source),
    ]);

    const { stdout, stderr } = await execFile("cargo", ["run"], { cwd: dir });
    return [stdout, stderr];
  });
}

async function withScratchDir<T>(
  source: string,
  cargofile: string,
  f: (dir: string) => Promise<T>
): Promise<T> {
  const hash = createHash("sha256");
  hash.update(source).update(cargofile);
  const digest = hash.digest("hex");

  const scratchDir = path.join(
    tmpdir(),
    "rust-markdown-notebook-scratch",
    digest
  );

  console.log(scratchDir);
  await mkdir(scratchDir, { recursive: true });
  const result = await f(scratchDir);
  await rm(scratchDir, { recursive: true, force: true });

  return result;
}
