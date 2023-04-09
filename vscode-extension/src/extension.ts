import * as vscode from "vscode";
import { Serializer } from "./serializer";
import { Controller } from "./controller";

const providerOptions = {} satisfies vscode.NotebookDocumentContentOptions;

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

  context.subscriptions.push(new Controller());

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
