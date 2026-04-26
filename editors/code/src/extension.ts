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
    return process.env["TF2_VSCRIPT_LS_DEV"] === "1";
}

function getLocalServerPath(): string {
    const isWindows = process.platform == "win32";
    if (inDebug()) {
        return path.join("..", "..", "target", "debug", isWindows ? "tf2-vscript-ls.exe" : "tf2-vscript-ls");
    }
    return path.join("server", isWindows ? "tf2-vscript-ls.exe" : "tf2-vscript-ls");
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
    const config = vscode.workspace.getConfiguration('tf2vscript');
    await config.update('tf2Root', selectedPath, vscode.ConfigurationTarget.Global);
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

    const config = vscode.workspace.getConfiguration('tf2vscript');
    const tf2Root = config.get<string>('tf2Root') ?? '';

    if (!tf2Root) {
        vscode.window.showWarningMessage(
            'TF2 VScript: TF2Root is not set. Imports will not work.',
            'Select Directory',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Select Directory') {
                selectTF2Root();
            } else if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'TF2Vscript.TF2Root');
            }
        });
    }

    const stdlibPath = inDebug() ?
        path.join(context.extensionPath, "..", "..", "vscript_lib") :
        path.join(context.extensionPath, "vscript_lib");

    const serverOptions: ServerOptions = { run, debug: run };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'tf2vscript' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.nut')
        },
        initializationOptions: {
            tf2Root,
            builtinsPath: path.join(stdlibPath, "builtins.nut"),
            squirrelLibPath: path.join(stdlibPath, "squirrel.nut"),
            vscriptLibPath: path.join(stdlibPath, "vscript.nut"),
        }
    };

    client = new LanguageClient(
        'tf2-vscript-language-server',
        'TF2 VScript Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    // Register the command
    context.subscriptions.push(
        vscode.commands.registerCommand('tf2vscript.selectTF2Root', selectTF2Root)
    );

    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('tf2vscript.tf2Root')) {
                client.sendNotification('workspace/didChangeConfiguration', {
                    settings: vscode.workspace.getConfiguration('tf2vscript')
                });
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