import * as vscode from "vscode";
import * as wasm from "rust_markdown_notebook_wasm";

const providerOptions = {} satisfies vscode.NotebookDocumentContentOptions;

// there are globals in workers and nodejs
declare class TextDecoder {
  decode(data: Uint8Array): string;
}
declare class TextEncoder {
  encode(data: string): Uint8Array;
}

export function activate(context: vscode.ExtensionContext) {
  console.log(
    'Congratulations, your extension "rust-markdown-notebook" is now active!'
  );

  context.subscriptions.push(
    vscode.workspace.registerNotebookSerializer(
      "rust-markdown-notebook",
      new Serializer(),
      providerOptions
    )
  );

  // The command has been defined in the package.json file
  // Now provide the implementation of the command with registerCommand
  // The commandId parameter must match the command field in package.json
  let disposable = vscode.commands.registerCommand(
    "rust-markdown-notebook.helloWorld",
    () => {
      // The code you place here will be executed every time your command is executed
      // Display a message box to the user
      vscode.window.showInformationMessage(
        "Hello World from vscode-extension!"
      );
    }
  );

  context.subscriptions.push(disposable);
}

// This method is called when your extension is deactivated
export function deactivate() {}

var debugContent: Uint8Array;

class Serializer implements vscode.NotebookSerializer {
  deserializeNotebook(
    content: Uint8Array,
    token: vscode.CancellationToken
  ): vscode.NotebookData | Thenable<vscode.NotebookData> {
    // TODO
    debugContent = content;

    const textDecoder = new TextDecoder();
    const source = textDecoder.decode(content);
    const notebook = wasm.to_notebook(source);

    const cells = JSON.parse(notebook).cells.map((cell: any) => {
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
    return debugContent;
  }
}
