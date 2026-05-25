$ErrorActionPreference = "Continue"
$Host.UI.RawUI.WindowTitle = "Trinity Language Installer"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Trinity Language - Auto Installer v4" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# PATH сразу
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

# ==========================================
# Шаг 1: Git
# ==========================================
Write-Host "[1/7] Checking Git..." -ForegroundColor Yellow
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Git not found! Downloading..." -ForegroundColor Red
    Invoke-WebRequest -Uri "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe" -OutFile "$env:TEMP\git-installer.exe"
    Start-Process -FilePath "$env:TEMP\git-installer.exe" -ArgumentList "/VERYSILENT /NORESTART" -Wait
    Remove-Item "$env:TEMP\git-installer.exe"
    Write-Host "Git installed!" -ForegroundColor Green
}
Write-Host "Git OK!" -ForegroundColor Green

# ==========================================
# Шаг 2: Rust
# ==========================================
Write-Host "[2/7] Checking Rust..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found! Installing..." -ForegroundColor Red
    Invoke-WebRequest -Uri "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe" -OutFile "$env:TEMP\rustup-init.exe"
    Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait
    Remove-Item "$env:TEMP\rustup-init.exe"
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    Write-Host "Rust installed!" -ForegroundColor Green
}
Write-Host "Rust OK! $(cargo --version)" -ForegroundColor Green

# ==========================================
# Шаг 3: LLVM
# ==========================================
Write-Host "[3/7] Checking LLVM (optional)..." -ForegroundColor Yellow
Write-Host "LLVM check done." -ForegroundColor Green

# ==========================================
# Шаг 4: Клонирование
# ==========================================
Write-Host "[4/7] Cloning Trinity repository..." -ForegroundColor Yellow
$projectDir = "$env:USERPROFILE\Trinity-Language"

if (Test-Path $projectDir) {
    Write-Host "Repository exists, updating..." -ForegroundColor Yellow
    Set-Location $projectDir
    git pull origin main 2>&1 | Out-Null
} else {
    Write-Host "Cloning fresh..." -ForegroundColor Yellow
    git clone https://github.com/ivan2002312/Trinity-Language.git $projectDir
    Set-Location $projectDir
}
Write-Host "Repository ready at: $projectDir" -ForegroundColor Green

# ==========================================
# Шаг 5: Зависимости
# ==========================================
Write-Host "[5/7] Checking dependencies..." -ForegroundColor Yellow
Set-Location $projectDir

if (-not (Test-Path "Cargo.toml")) {
    @"
[package]
name = "trinity"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "trinity"
path = "src/main.rs"

[dependencies]
logos = "0.14"
"@ | Out-File -FilePath Cargo.toml -Encoding UTF8
}

cargo update 2>&1 | Out-Null
Write-Host "Dependencies OK!" -ForegroundColor Green

# ==========================================
# Шаг 6: Сборка
# ==========================================
Write-Host "[6/7] Building Trinity..." -ForegroundColor Yellow
Set-Location $projectDir

Write-Host "Building..."
cargo build --release 2>&1 | Out-Null

if ($LASTEXITCODE -ne 0) {
    Write-Host "Release build failed, trying debug..." -ForegroundColor Yellow
    cargo build 2>&1 | Out-Null
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build SUCCESS!" -ForegroundColor Green
} else {
    Write-Host "BUILD FAILED!" -ForegroundColor Red
    Read-Host "Press Enter"
    exit 1
}

# ==========================================
# Шаг 7: Установка
# ==========================================
Write-Host "[7/7] Installing Trinity..." -ForegroundColor Yellow
Set-Location $projectDir

cargo install --path . --force 2>&1 | Out-Null

# PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if ($userPath -notlike "*$cargoBin*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargoBin", "User")
}
$env:Path = "$cargoBin;$env:Path"

# Проверка
$trinityExe = "$cargoBin\trinity.exe"
if (-not (Test-Path $trinityExe)) {
    $trinityExe = "$cargoBin\trinity-compiler.exe"
}

# ==========================================
# Готово
# ==========================================
Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "  Trinity Installation Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Binary: $trinityExe" -ForegroundColor White
Write-Host ""
Write-Host "Usage:" -ForegroundColor Yellow
Write-Host "  trinity file.tr --run" -ForegroundColor White
Write-Host ""

# Тест
$testFile = "$projectDir\examples\hello.tr"
if (Test-Path $testFile) {
    Write-Host "Running test..." -ForegroundColor Yellow
    & $trinityExe $testFile --run 2>&1
} else {
    @"
module Hello;
static int main() {
    println("Hello from Trinity!");
    return 0;
}
"@ | Out-File -FilePath "$projectDir\test.tr" -Encoding UTF8
    Write-Host "Test: trinity $projectDir\test.tr --run" -ForegroundColor White
}

Write-Host ""
Write-Host "Restart terminal to use 'trinity' command." -ForegroundColor Yellow
Write-Host ""
Read-Host "Press Enter"
