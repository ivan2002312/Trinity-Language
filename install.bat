@echo off
echo === Trinity Compiler Installer ===
echo.

:: ???????? Rust
where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing Rust...
    curl -sSf -o rustup-init.exe https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
    rustup-init.exe -y
    del rustup-init.exe
)

:: ??????????? ??????
echo Copying project...
if not exist "%USERPROFILE%\Trinity" mkdir "%USERPROFILE%\Trinity"
xcopy /E /Y "%~dp0*" "%USERPROFILE%\Trinity\" >nul

:: ??????
cd /d "%USERPROFILE%\Trinity"
echo Building...
cargo build --release

:: ?????????? ? PATH
if %errorlevel% equ 0 (
    echo Adding to PATH...
    setx PATH "%PATH%;%USERPROFILE%\Trinity\target\release" >nul
    echo Done! Restart terminal and run: trinity file.tr --run
) else (
    echo Build failed. Run: cargo build
)

pause
