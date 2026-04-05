import * as vscode from 'vscode';
import * as path from 'path';
import {
    ConnectionError,
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
} from 'vscode-languageclient/node';
import { workspace } from 'vscode';

let client: LanguageClient;

function inDebug(): boolean {
    return process.env["TF2_VSCRIPT_LS_DEV"] === "1"
}

function getLocalServerPath(): string {
    const isWindows = process.platform == "win32";

    if (inDebug()) {
        return path.join("..", "..", "target", "debug", isWindows ? "tf2-vscript-ls.exe" : "tf2-vscript-ls");
    }

    return path.join("server", isWindows ? "tf2-vscript-ls.exe" : "tf2-vscript-ls");
}

export function activate(context: vscode.ExtensionContext) {
    const serverPath = context.asAbsolutePath(getLocalServerPath());

    const env = { ...process.env };
    if (inDebug()) {
        env.RUST_LOG = "debug"
    }

    const run: Executable = {
        command: serverPath,
        options: { env },
    };

    const stdlibPath = inDebug() ?
        path.join(context.extensionPath, "..", "..", "vscript_lib") :
        path.join(context.extensionPath, "vscript_lib");

    const serverOptions: ServerOptions = {
        run,
        debug: run,
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'tf2vscript' }],
        synchronize: {
            fileEvents: workspace.createFileSystemWatcher('**/*.nut')
        },
        initializationOptions: {
            tf2Root: "/mnt/d/program files/steam/steamapps/common/team fortress 2",
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

    if (inDebug()) {
        client.traceOutputChannel.show(true);
    }
}

export function deactivate(): Thenable<void> | undefined {
    return client?.stop();
}
