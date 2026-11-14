# Compila o núcleo em Rust para WebAssembly e coloca o resultado em /wasm (na raiz do site).
# Requer: rustup target add wasm32-unknown-unknown  e  cargo install wasm-bindgen-cli --version 0.2.100
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
cargo test --quiet
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir ..\..\wasm target\wasm32-unknown-unknown\release\receitas.wasm
Write-Host "Pronto: wasm\receitas.js e wasm\receitas_bg.wasm atualizados."
