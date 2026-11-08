#Requires -Version 5.1
$ErrorActionPreference = "Stop"

Set-Location $PSScriptRoot

# Adiciona ~/.cargo/bin ao PATH se existir
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if ((Test-Path $cargoBin) -and ($env:PATH -notlike "*$cargoBin*")) {
    $env:PATH = "$cargoBin;$env:PATH"
}

# Se o Cargo estiver instalado, usa o servidor HTTP nativo em Rust na porta 8080
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "Iniciando servidor HTTP nativo em Rust em http://localhost:8080 ..." -ForegroundColor Green
    Start-Process "http://localhost:8080"
    cargo run --release
    exit 0
}

# Caso o Rust/Cargo nao esteja instalado, sobe um servidor HTTP nativo do .NET/PowerShell na porta 8080
# servindo o web/index.html e web/minecraft_rust.wasm pre-compilados!
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  MINECRAFTO-RUST (Modo Instantaneo Windows sem Cargo)      " -ForegroundColor Green
Write-Host "  Servindo motor Rust WebAssembly pre-compilado + WebGL2    " -ForegroundColor Cyan
Write-Host "  URL: http://localhost:8080                                " -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan

$listener = New-Object System.Net.HttpListener
$listener.Prefixes.Add("http://localhost:8080/")
try {
    $listener.Start()
    Start-Process "http://localhost:8080"
    Write-Host "Servidor ativo! Pressione Ctrl+C para encerrar." -ForegroundColor Green

    while ($listener.IsListening) {
        $context = $listener.GetContext()
        $reqUrl = $context.Request.Url.AbsolutePath
        $res = $context.Response

        if ($reqUrl -eq "/minecraft_rust.wasm") {
            $bytes = [System.IO.File]::ReadAllBytes((Join-Path $PSScriptRoot "web\minecraft_rust.wasm"))
            $res.ContentType = "application/wasm"
            $res.ContentLength64 = $bytes.Length
            $res.OutputStream.Write($bytes, 0, $bytes.Length)
        } elseif ($reqUrl -eq "/api/status") {
            $json = '{"engine":"minecrafto-rust","target":"wasm32-unknown-unknown + WebGL2","chunks":"8x8","chunk_size":16,"world_dimensions":[128,64,128],"total_voxels":1048576}'
            $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
            $res.ContentType = "application/json"
            $res.ContentLength64 = $bytes.Length
            $res.OutputStream.Write($bytes, 0, $bytes.Length)
        } else {
            $bytes = [System.IO.File]::ReadAllBytes((Join-Path $PSScriptRoot "web\index.html"))
            $res.ContentType = "text/html; charset=utf-8"
            $res.ContentLength64 = $bytes.Length
            $res.OutputStream.Write($bytes, 0, $bytes.Length)
        }
        $res.Close()
    }
} catch {
    Write-Host "Porta 8080 ocupada ou indisponivel. Abrindo index.html diretamente no navegador..." -ForegroundColor Yellow
    Start-Process (Join-Path $PSScriptRoot "index.html")
} finally {
    if ($listener.IsListening) {
        $listener.Stop()
    }
}
