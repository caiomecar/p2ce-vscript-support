import * as vscode from 'vscode';
import * as path from 'path';
import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient/node';
import { workspace } from 'vscode';

let client: LanguageClient;

function inDebug(): boolean {
    return process.env["P2CE_VSCRIPT_LS_DEV"] === "1";
}

function getLocalServerPath(): string {
    const isWindows = process.platform == "win32";
    if (inDebug()) {
        return path.join("..", "..", "target", "debug", isWindows ? "p2ce-vscript-ls.exe" : "p2ce-vscript-ls");
    }
    return path.join("server", isWindows ? "p2ce-vscript-ls.exe" : "p2ce-vscript-ls");
}

async function selectTF2Root() {
    const result = await vscode.window.showOpenDialog({
        canSelectFiles: false,
        canSelectFolders: true,
        canSelectMany: false,
        openLabel: 'Select TF2 Root Directory',
        title: 'TF2 VScript: Select TF2 Root',
    });

    if (!result || result.length === 0) {
        return;
    }

    const selectedPath = result[0].fsPath;
    const config = vscode.workspace.getConfiguration('p2ce_vscript');
    await config.update('tf2RootPath', selectedPath, vscode.ConfigurationTarget.Global);
    vscode.window.showInformationMessage(`TF2 VScript: TF2Root set to "${selectedPath}"`);
}

export function activate(context: vscode.ExtensionContext) {
    const serverPath = context.asAbsolutePath(getLocalServerPath());
    const env = { ...process.env };
    if (inDebug()) {
        env.RUST_LOG = "debug";
    }

    const run: Executable = {
        command: serverPath,
        options: { env },
    };

    const config = vscode.workspace.getConfiguration('p2ce_vscript');
    const tf2RootPath = config.get<string>('tf2RootPath') ?? '';

    if (!tf2RootPath) {
        vscode.window.showWarningMessage(
            'TF2 VScript: TF2Root is not set. Imports will not work.',
            'Select Directory',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Select Directory') {
                selectTF2Root();
            } else if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'p2ce_vscript.tf2RootPath');
            }
        });
    }

    const unusedVariables = config.get<string>('unusedVariables') ?? 'hint';
    const unreachableCode = config.get<string>('unreachableCode') ?? 'warn';
    const typeHints = config.get<boolean>('inlayHints.typeHints') ?? true;
    const parameterHints = config.get<boolean>('inlayHints.parameterHints') ?? true;
    const enumMemberValue = config.get<boolean>('inlayHints.enumMemberValue') ?? true;
    const workspaceDiagnostics = config.get<boolean>('workspaceDiagnostics') ?? false;

    const stdlibPath = inDebug() ?
        path.join(context.extensionPath, "..", "..", "vscript_lib") :
        path.join(context.extensionPath, "vscript_lib");

    const serverOptions: ServerOptions = { run, debug: run };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'p2ce_vscript' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.nut')
        },
        initializationOptions: {
            builtinsPath: path.join(stdlibPath, "builtins.nut"),
            squirrelLibPath: path.join(stdlibPath, "squirrel.nut"),
            vscriptLibPath: path.join(stdlibPath, "vscript.nut"),
            tf2RootPath,
            unusedVariables,
            unreachableCode,
            inlayHints: {
                typeHints,
                enumMemberValue,
                parameterHints,
            },
            workspaceDiagnostics,
        }
    };

    client = new LanguageClient(
        'p2ce-vscript-language-server',
        'P2CE VScript Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    context.subscriptions.push(
        vscode.commands.registerCommand('p2ce_vscript.selectTF2Root', selectTF2Root)
    );

    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(async e => {
            if (e.affectsConfiguration('p2ce_vscript')) {
                client.sendNotification('workspace/didChangeConfiguration', {
                    settings: vscode.workspace.getConfiguration('p2ce_vscript')
                });
            }

            if (e.affectsConfiguration('p2ce_vscript.tf2RootPath')
                || e.affectsConfiguration('p2ce_vscript.unusedVariables')
                || e.affectsConfiguration('p2ce_vscript.unreachableCode')) {
                vscode.window.showInformationMessage(
                    'TF2 VScript: Edit a file to refresh diagnostics with the new settings.'
                );
            }
        })
    );

    if (inDebug()) {
        client.traceOutputChannel.show(true);
    }
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}