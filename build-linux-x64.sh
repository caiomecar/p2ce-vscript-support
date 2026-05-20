python3 ./gen/dataparser.py 
cargo build --release
cp -r ./vscript_lib ./editors/code/
mkdir -p ./editors/code/server && cp ./target/release/p2ce-vscript-ls ./editors/code/server/
cp ./LICENSE ./editors/code/
cp ./CHANGELOG.md ./editors/code/
cp ./README.md ./editors/code/
cd ./editors/code
npm ci
npm run build-release
npx vsce package --target linux-x64