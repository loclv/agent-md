import * as vscode from 'vscode';
import { spawn } from 'child_process';
import * as path from 'path';

interface FormatOptions {
	removeBold: boolean;
	compactBlankLines: boolean;
	collapseSpaces: boolean;
	removeHorizontalRules: boolean;
	removeEmphasis: boolean;
}

function getFormatOptions(): FormatOptions {
	const config = vscode.workspace.getConfiguration('agentMd.format');
	return {
		removeBold: config.get<boolean>('removeBold', true),
		compactBlankLines: config.get<boolean>('compactBlankLines', true),
		collapseSpaces: config.get<boolean>('collapseSpaces', true),
		removeHorizontalRules: config.get<boolean>('removeHorizontalRules', true),
		removeEmphasis: config.get<boolean>('removeEmphasis', true),
	};
}

function getAgentMdPath(): string {
	const config = vscode.workspace.getConfiguration('agentMd');
	return config.get<string>('path', 'agent-md');
}

function formatWithAgentMd(
	document: vscode.TextDocument,
	options: FormatOptions
): Promise<string> {
	return new Promise((resolve, reject) => {
		const agentMdPath = getAgentMdPath();
		const filePath = document.fileName;

		const args: string[] = ['fmt', filePath];

		if (!options.removeBold) {
			args.push('--remove-bold=false');
		}
		if (!options.compactBlankLines) {
			args.push('--compact-blank-lines=false');
		}
		if (!options.collapseSpaces) {
			args.push('--collapse-spaces=false');
		}
		if (!options.removeHorizontalRules) {
			args.push('--remove-horizontal-rules=false');
		}
		if (!options.removeEmphasis) {
			args.push('--remove-emphasis=false');
		}

		const process = spawn(agentMdPath, args);

		let stdout = '';
		let stderr = '';

		process.stdout.on('data', (data) => {
			stdout += data.toString();
		});

		process.stderr.on('data', (data) => {
			stderr += data.toString();
		});

		process.on('close', async (code) => {
			if (code === 0) {
				// Read the formatted file content
				try {
					const doc = await vscode.workspace.openTextDocument(filePath);
					resolve(doc.getText());
				} catch (err) {
					reject(err);
				}
			} else {
				reject(new Error(`agent-md fmt failed with code ${code}: ${stderr}`));
			}
		});

		process.on('error', (err) => {
			if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
				reject(new Error(`agent-md executable not found at "${agentMdPath}". Please ensure it's installed and in your PATH, or configure the correct path in settings.`));
			} else {
				reject(err);
			}
		});
	});
}

class AgentMdFormatter implements vscode.DocumentFormattingEditProvider {
	async provideDocumentFormattingEdits(
		document: vscode.TextDocument,
		_options: vscode.FormattingOptions,
		_token: vscode.CancellationToken
	): Promise<vscode.TextEdit[] | undefined> {
		const formatOptions = getFormatOptions();

		try {
			const originalContent = document.getText();
			await formatWithAgentMd(document, formatOptions);

			// Re-read the document to get formatted content
			const formattedDoc: vscode.TextDocument = await vscode.workspace.openTextDocument(document.fileName);
			const formattedContent = formattedDoc.getText();

			if (formattedContent === originalContent) {
				return undefined;
			}

			const fullRange = new vscode.Range(
				document.positionAt(0),
				document.positionAt(originalContent.length)
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
		{ language: 'markdown', scheme: 'file' },
		formatter
	);

	context.subscriptions.push(disposable);

	// Also register for range formatting (format selection)
	const rangeDisposable = vscode.languages.registerDocumentRangeFormattingEditProvider(
		{ language: 'markdown', scheme: 'file' },
		{
			async provideDocumentRangeFormattingEdits(
				document: vscode.TextDocument,
				range: vscode.Range,
				options: vscode.FormattingOptions,
				token: vscode.CancellationToken
			): Promise<vscode.TextEdit[] | undefined> {
				// For range formatting, we still format the whole document
				// but only return the edits for the selected range
				const edits = await formatter.provideDocumentFormattingEdits(document, options, token);
				if (!edits) {
					return undefined;
				}

				// Filter edits to only those within the range
				return edits.filter(edit => {
					const editRange = edit.range;
					return editRange.start.isAfterOrEqual(range.start) &&
						   editRange.end.isBeforeOrEqual(range.end);
				});
			}
		}
	);

	context.subscriptions.push(rangeDisposable);
}

export function deactivate() {}
