# setup.ps1 - Trinity Language Installer

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Trinity Language - Installer" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# 1. Проверка Git
Write-Host "[1/4] Checking Git..." -ForegroundColor Yellow
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Git not found! Install from: https://git-scm.com" -ForegroundColor Red
    Start-Process "https://git-scm.com/download/win"
    Read-Host "Press Enter after installing Git"
}
Write-Host "Git OK!" -ForegroundColor Green

# 2. Проверка Rust
Write-Host "[2/4] Checking Rust..." -ForegroundColor Yellow
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found! Installing..." -ForegroundColor Red
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}
Write-Host "Rust OK! ($(cargo --version))" -ForegroundColor Green

# 3. Клонирование
Write-Host "[3/4] Cloning Trinity..." -ForegroundColor Yellow
$dir = "$env:USERPROFILE\Trinity-Language"
if (Test-Path $dir) {
    Set-Location $dir
    git pull
} else {
    git clone https://github.com/ivan2002312/Trinity-Language.git $dir
    Set-Location $dir
}
Write-Host "Repository ready!" -ForegroundColor Green

# 4. Сборка и установка
Write-Host "[4/4] Building & Installing..." -ForegroundColor Yellow
cargo build --release
cargo install --path $env:USERPROFILE\Trinity-Language

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  Trinity Installed!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Usage: trinity file.tr --run" -ForegroundColor White
Write-Host ""
Write-Host "Test:" -ForegroundColor Yellow
Write-Host "  cd $dir" -ForegroundColor White
Write-Host "  trinity examples\hello.tr --run" -ForegroundColor White
Write-Host ""

Read-Host "Press Enter"
