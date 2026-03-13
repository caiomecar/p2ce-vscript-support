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

function getLocalServerPath(): string {
    const isWindows = process.platform == "win32";

    if (process.env["TF2_VSCRIPT_LS_DEV"] === "1") {
        return path.join("..", "..", "target", "debug", isWindows ? "tf2-vscript-ls.exe" : "tf2-vscript-ls");
    }

    return path.join("out", isWindows ? "server.exe" : "server");
}

export function activate(context: vscode.ExtensionContext) {
    const serverPath = context.asAbsolutePath(getLocalServerPath());

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
