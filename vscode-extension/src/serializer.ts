import * as vscode from "vscode";
import * as wasm from "rust-markdown-notebook";

// there are globals in workers and nodejs
declare class TextDecoder {
  decode(data: Uint8Array): string;
}
declare class TextEncoder {
  encode(data: string): Uint8Array;
}

export var debugContent: Uint8Array;

export class Serializer implements vscode.NotebookSerializer {
  deserializeNotebook(
    content: Uint8Array,
    token: vscode.CancellationToken
  ): vscode.NotebookData | Thenable<vscode.NotebookData> {
    // TODO
    debugContent = content;

    const textDecoder = new TextDecoder();
    const source = textDecoder.decode(content);
    const notebook = wasm.to_notebook(source);

    const cells = notebook.cells.map((cell: any) => {
      const [kind, languageId] =
        cell.kind === "Markdown"
          ? [vscode.NotebookCellKind.Markup, "markdown"]
          : [vscode.NotebookCellKind.Code, "rust"];

      const value =
        cell.kind === "EvaluatedRustCode" ? cell.cell.source : cell.cell;

      return { kind, value, languageId };
    });

    return { cells };
  }

  serializeNotebook(
    data: vscode.NotebookData,
    token: vscode.CancellationToken
  ): Uint8Array | Thenable<Uint8Array> {
    // TODO
    return debugContent;
  }
}
