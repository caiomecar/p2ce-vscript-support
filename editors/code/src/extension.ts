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

export function activate(context: vscode.ExtensionContext) {
    const serverPath = context.asAbsolutePath(
        path.join("..", "..", "target", "debug", "tf2-vscript-lsp")
    );

    const run: Executable = {
        command: serverPath,
        options: {
            env: {
                ...process.env,
                RUST_LOG: "debug",
            },
        },
    };


    const serverOptions: ServerOptions = {
        run,
        debug: run,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'tf2vscript' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.nut')
        }
    };

    client = new LanguageClient(
        'tf2-vscript-language-server',
        'TF2 VScript Language Server',
        serverOptions,
        clientOptions
    );

    client.start();

    client.traceOutputChannel.show(true);
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
