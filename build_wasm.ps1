#Requires -Version 5.1
$ErrorActionPreference = "Stop"

# Garante que o diretório ~/.cargo/bin esteja no PATH caso o Rust tenha acabado de ser instalado
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $cargoBin) -and ($env:PATH -notlike "*$cargoBin*")) {
    $env:PATH = "$cargoBin;$env:PATH"
}

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "==================================================================" -ForegroundColor Yellow
    Write-Host " [AVISO] O compilador Rust (rustc/cargo) nao foi encontrado no Windows." -ForegroundColor Yellow
    Write-Host " O repositorio ja inclui 'web/minecraft_rust.wasm' e 'web/index.html'" -ForegroundColor Cyan
    Write-Host " pre-compilados! Para jogar imediatamente sem instalar o Rust, rode:" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "   .\start_windows.ps1" -ForegroundColor Green
    Write-Host ""
    Write-Host " Ou para instalar o Rust no Windows PowerShell, execute:" -ForegroundColor White
    Write-Host "   winget install Rustlang.Rustup" -ForegroundColor Green
    Write-Host "==================================================================" -ForegroundColor Yellow
    exit 0
}

if (Get-Command rustup -ErrorAction SilentlyContinue) {
    Write-Host "[0/3] Garantindo target wasm32-unknown-unknown no rustup..." -ForegroundColor Cyan
    rustup target add wasm32-unknown-unknown
}

Write-Host "[1/3] Compilando motor voxel Rust (src/lib.rs) -> web/minecraft_rust.wasm..." -ForegroundColor Cyan
$env:RUSTC_BOOTSTRAP = "1"
rustc --target wasm32-unknown-unknown --crate-type cdylib -O src/lib.rs -o web/minecraft_rust.wasm

Write-Host "[2/3] Empacotando binario WebAssembly em web/index.html e index.html (nativo PowerShell)..." -ForegroundColor Cyan
$wasmBytes = [System.IO.File]::ReadAllBytes((Resolve-Path "web/minecraft_rust.wasm"))
$wasmB64 = [System.Convert]::ToBase64String($wasmBytes)
$tpl = [System.IO.File]::ReadAllText((Resolve-Path "web/template.html"), [System.Text.Encoding]::UTF8)
$out = $tpl.Replace("__WASM_BASE64_PLACEHOLDER__", $wasmB64)

$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText((Join-Path $PSScriptRoot "web\index.html"), $out, $utf8NoBom)
[System.IO.File]::WriteAllText((Join-Path $PSScriptRoot "index.html"), $out, $utf8NoBom)

$outKb = [math]::Round($out.Length / 1024, 1)
$b64Kb = [math]::Round($wasmB64.Length / 1024, 1)
Write-Host "Gerado web/index.html ($outKb KB) com Rust WASM embutido ($b64Kb KB base64)" -ForegroundColor Green

Write-Host "[3/3] Executando testes unitarios nativos em Rust (cargo test)..." -ForegroundColor Cyan
cargo test
