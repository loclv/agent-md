import * as path from "node:path";
import * as vscode from "vscode";
import { formatContent, type FormatOptions } from "./formatter";
import { resolveAgentMdPath } from "./resolver";

function getFormatOptions(): FormatOptions {
  const config = vscode.workspace.getConfiguration("agentMd.format");
  return {
    removeBold: config.get<boolean>("removeBold", true),
    compactBlankLines: config.get<boolean>("compactBlankLines", true),
    collapseSpaces: config.get<boolean>("collapseSpaces", true),
    removeHorizontalRules: config.get<boolean>("removeHorizontalRules", true),
    removeEmphasis: config.get<boolean>("removeEmphasis", true),
  };
}

function getAgentMdPath(document?: vscode.TextDocument): string {
  const config = vscode.workspace.getConfiguration("agentMd");
  const configuredPath = config.get<string>("path", "agent-md");

  const workspaceFolders: string[] = [];
  if (document) {
    const docFolder = vscode.workspace.getWorkspaceFolder(document.uri);
    if (docFolder) {
      workspaceFolders.push(docFolder.uri.fsPath);
    }
  }
  if (vscode.workspace.workspaceFolders) {
    for (const folder of vscode.workspace.workspaceFolders) {
      if (!workspaceFolders.includes(folder.uri.fsPath)) {
        workspaceFolders.push(folder.uri.fsPath);
      }
    }
  }

  return resolveAgentMdPath({
    configuredPath,
    workspaceFolders,
    documentPath: document?.uri.scheme === "file" ? document.uri.fsPath : undefined,
  });
}

function formatWithAgentMd(
  content: string,
  options: FormatOptions,
  document?: vscode.TextDocument,
): Promise<string> {
  const agentMdPath = getAgentMdPath(document);
  const cwd =
    document?.uri.scheme === "file"
      ? path.dirname(document.uri.fsPath)
      : vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;

  return formatContent(content, agentMdPath, options, cwd);
}

class AgentMdFormatter implements vscode.DocumentFormattingEditProvider {
  async provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    _options: vscode.FormattingOptions,
    _token: vscode.CancellationToken,
  ): Promise<vscode.TextEdit[] | undefined> {
    const formatOptions = getFormatOptions();

    try {
      const originalContent = document.getText();
      const formattedContent = await formatWithAgentMd(originalContent, formatOptions, document);

      if (formattedContent === originalContent) {
        return undefined;
      }

      const fullRange = new vscode.Range(
        document.positionAt(0),
        document.positionAt(originalContent.length),
      );

      return [vscode.TextEdit.replace(fullRange, formattedContent)];
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      vscode.window.showErrorMessage(`Failed to format with agent-md: ${errorMessage}`);
      return undefined;
    }
  }
}

export function activate(context: vscode.ExtensionContext) {
  const formatter = new AgentMdFormatter();

  const disposable = vscode.languages.registerDocumentFormattingEditProvider(
    { language: "markdown", scheme: "file" },
    formatter,
  );

  context.subscriptions.push(disposable);

  // Also register for range formatting (format selection)
  const rangeDisposable = vscode.languages.registerDocumentRangeFormattingEditProvider(
    { language: "markdown", scheme: "file" },
    {
      async provideDocumentRangeFormattingEdits(
        document: vscode.TextDocument,
        range: vscode.Range,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken,
      ): Promise<vscode.TextEdit[] | undefined> {
        // For range formatting, we still format the whole document
        // but only return the edits for the selected range
        const edits = await formatter.provideDocumentFormattingEdits(document, options, token);
        if (!edits) {
          return undefined;
        }

        // Filter edits to only those within the range
        return edits.filter((edit) => {
          const editRange = edit.range;
          return (
            editRange.start.isAfterOrEqual(range.start) && editRange.end.isBeforeOrEqual(range.end)
          );
        });
      },
    },
  );

  context.subscriptions.push(rangeDisposable);
}

export function deactivate() {}
