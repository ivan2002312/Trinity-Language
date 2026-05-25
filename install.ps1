

$ErrorActionPreference = "Stop"
$Host.UI.RawUI.WindowTitle = "Trinity Language Installer"

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Trinity Language - Auto Installer v3" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ==========================================
# Шаг 1: Проверка и установка Git
# ==========================================
Write-Host "[1/7] Checking Git..." -ForegroundColor Yellow
$gitExists = Get-Command git -ErrorAction SilentlyContinue
if (-not $gitExists) {
    Write-Host "Git not found! Downloading..." -ForegroundColor Red
    $gitUrl = "https://github.com/git-for-windows/git/releases/download/v2.43.0.windows.1/Git-2.43.0-64-bit.exe"
    $gitInstaller = "$env:TEMP\git-installer.exe"
    Invoke-WebRequest -Uri $gitUrl -OutFile $gitInstaller
    Start-Process -FilePath $gitInstaller -ArgumentList "/VERYSILENT /NORESTART" -Wait
    Remove-Item $gitInstaller
    Write-Host "Git installed!" -ForegroundColor Green
}
Write-Host "Git OK!" -ForegroundColor Green

# ==========================================
# Шаг 2: Проверка и установка Rust
# ==========================================
Write-Host "[2/7] Checking Rust..." -ForegroundColor Yellow
$rustExists = Get-Command rustc -ErrorAction SilentlyContinue
if (-not $rustExists) {
    Write-Host "Rust not found! Installing..." -ForegroundColor Red
    $rustUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $rustInstaller = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustUrl -OutFile $rustInstaller
    Start-Process -FilePath $rustInstaller -ArgumentList "-y --default-toolchain stable" -Wait
    Remove-Item $rustInstaller
    Write-Host "Rust installed!" -ForegroundColor Green
}

# Обновление PATH для текущей сессии
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
Write-Host "Rust OK!" -ForegroundColor Green
cargo --version

# ==========================================
# Шаг 3: Проверка LLVM
# ==========================================
Write-Host "[3/7] Checking LLVM (optional)..." -ForegroundColor Yellow
$llvmExists = Get-Command llvm-config -ErrorAction SilentlyContinue
if (-not $llvmExists) {
    Write-Host "LLVM not found (optional, for JIT compilation)" -ForegroundColor DarkGray
}
Write-Host "LLVM check done." -ForegroundColor Green

# ==========================================
# Шаг 4: Клонирование репозитория
# ==========================================
Write-Host "[4/7] Cloning Trinity repository..." -ForegroundColor Yellow
$projectDir = "$env:USERPROFILE\Trinity-Language"

if (Test-Path $projectDir) {
    Write-Host "Repository exists, updating..." -ForegroundColor Yellow
    Set-Location $projectDir
    git pull origin main
} else {
    Write-Host "Cloning fresh..." -ForegroundColor Yellow
    git clone https://github.com/ivan2002312/Trinity-Language.git $projectDir
    Set-Location $projectDir
}
Write-Host "Repository ready at: $projectDir" -ForegroundColor Green

# ==========================================
# Шаг 5: Проверка и установка зависимостей
# ==========================================
Write-Host "[5/7] Checking dependencies..." -ForegroundColor Yellow
Set-Location $projectDir

if (-not (Test-Path "Cargo.toml")) {
    Write-Host "Cargo.toml not found! Creating..." -ForegroundColor Red
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

Write-Host "Updating dependencies..." -ForegroundColor Yellow
cargo update
Write-Host "Dependencies OK!" -ForegroundColor Green

# ==========================================
# Шаг 6: Сборка проекта
# ==========================================
Write-Host "[6/7] Building Trinity..." -ForegroundColor Yellow
Set-Location $projectDir

Write-Host "Cleaning old build..." -ForegroundColor DarkGray
cargo clean 2>$null

Write-Host "Building release version..." -ForegroundColor Yellow
$buildResult = cargo build --release 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Release build SUCCESS!" -ForegroundColor Green
    $buildOk = $true
} else {
    Write-Host "Release build failed, trying debug build..." -ForegroundColor Yellow
    $buildResult = cargo build 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Debug build SUCCESS!" -ForegroundColor Green
        $buildOk = $true
    } else {
        Write-Host "BUILD FAILED!" -ForegroundColor Red
        Write-Host $buildResult
        Read-Host "Press Enter to exit"
        exit 1
    }
}

# ==========================================
# Шаг 7: Установка
# ==========================================
Write-Host "[7/7] Installing Trinity..." -ForegroundColor Yellow
Set-Location $projectDir

$installResult = cargo install --path . 2>&1

if ($LASTEXITCODE -eq 0) {
    Write-Host "Trinity installed globally!" -ForegroundColor Green
} else {
    Write-Host "Global install failed, creating local batch file..." -ForegroundColor Yellow
    
    # Создаём bat-файл
    $trinityExe = "$projectDir\target\release\trinity.exe"
    if (Test-Path $trinityExe) {
        @"
@echo off
"$trinityExe" %*
"@ | Out-File -FilePath "$env:USERPROFILE\trinity.bat" -Encoding ASCII
        Write-Host "Local batch created: $env:USERPROFILE\trinity.bat" -ForegroundColor Green
    } else {
        $trinityExe = "$projectDir\target\debug\trinity.exe"
        if (Test-Path $trinityExe) {
            @"
@echo off
"$trinityExe" %*
"@ | Out-File -FilePath "$env:USERPROFILE\trinity.bat" -Encoding ASCII
            Write-Host "Local batch created: $env:USERPROFILE\trinity.bat" -ForegroundColor Green
        }
    }
}

# Добавление в PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if ($userPath -notlike "*$cargoBin*") {
    Write-Host "Adding to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargoBin", "User")
    $env:Path = "$env:Path;$cargoBin"
}

# ==========================================
# Готово!
# ==========================================
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Trinity Installation Complete!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Project: $projectDir" -ForegroundColor White
Write-Host "Binary:  $cargoBin\trinity.exe" -ForegroundColor White
Write-Host ""
Write-Host "Usage:" -ForegroundColor Yellow
Write-Host "  trinity file.tr --run" -ForegroundColor White
Write-Host "  trinity file.tr --lex-only" -ForegroundColor White
Write-Host "  trinity file.tr --parse-only" -ForegroundColor White
Write-Host ""

# ==========================================
# Тестовый запуск
# ==========================================
$testFile = "$projectDir\examples\hello.tr"
if (Test-Path $testFile) {
    Write-Host "Running test..." -ForegroundColor Yellow
    Write-Host ""
    Set-Location $projectDir
    
    # Пробуем запустить
    try {
        $trinityCmd = Get-Command trinity -ErrorAction Stop
        & trinity $testFile --run
    } catch {
        try {
            & "$projectDir\target\release\trinity.exe" $testFile --run
        } catch {
            try {
                & "$projectDir\target\debug\trinity.exe" $testFile --run
            } catch {
                Write-Host "Test skipped. Run manually:" -ForegroundColor Yellow
                Write-Host "  cd $projectDir" -ForegroundColor White
                Write-Host "  cargo run -- examples\hello.tr --run" -ForegroundColor White
            }
        }
    }
    Write-Host ""
} else {
    Write-Host "Creating test file..." -ForegroundColor Yellow
    @"
module Hello;

static int main() {
    var greeting = "Hello from Trinity!";
    println(greeting);
    var sum = 0;
    for (var i = 1; i <= 10; i = i + 1) {
        sum = sum + i;
    }
    println("Sum 1..10 = ", sum);
    return sum;
}
"@ | Out-File -FilePath "$projectDir\test.tr" -Encoding UTF8
    Write-Host "Test file created: $projectDir\test.tr" -ForegroundColor Green
    Write-Host "Run: trinity $projectDir\test.tr --run" -ForegroundColor White
}

Write-Host ""
Write-Host "IMPORTANT: Restart terminal for PATH to update!" -ForegroundColor Yellow
Write-Host ""
Read-Host "Press Enter to exit"
