python .\gen\dataparser.py
cargo build --release
Copy-Item -r ./vscript_lib ./editors/code/ -Force
Copy-Item -r ./target/release/p2ce-vscript-ls.exe ./editors/code/server/ -Force
Copy-Item -r ./LICENSE ./editors/code/ -Force
Copy-Item -r ./CHANGELOG.md ./editors/code/ -Force
Copy-Item -r ./README.md ./editors/code/ -Force
Set-Location ./editors/code
npm ci
npm run build-release
npx vsce package --target win32-x64