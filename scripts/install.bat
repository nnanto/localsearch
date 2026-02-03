@echo off
setlocal enabledelayedexpansion

:: ============================================================================
:: localsearch Installation Script for Windows (Batch File)
:: ============================================================================

:: Default parameters
set "INSTALL_DIR=%LOCALAPPDATA%\Programs\localsearch"
set "GITHUB_REPO=nnanto/localsearch"
set "SHOW_HELP=0"

:: Parse command line arguments
:parse_args
if "%~1"=="" goto end_parse
if /i "%~1"=="-h" set "SHOW_HELP=1"
if /i "%~1"=="--help" set "SHOW_HELP=1"
if /i "%~1"=="/?" set "SHOW_HELP=1"
if /i "%~1"=="-InstallDir" (
    set "INSTALL_DIR=%~2"
    shift
)
if /i "%~1"=="-GitHubRepo" (
    set "GITHUB_REPO=%~2"
    shift
)
shift
goto parse_args
:end_parse

:: Show help if requested
if "%SHOW_HELP%"=="1" (
    call :show_help
    exit /b 0
)

:: Main installation
echo [INFO] Installing localsearch CLI tool...
echo [INFO] Installation directory: %INSTALL_DIR%
echo [INFO] GitHub repository: %GITHUB_REPO%
echo.

call :install_localsearch
if errorlevel 1 (
    echo [ERROR] Installation failed!
    exit /b 1
)

echo [INFO] Installation completed successfully!
exit /b 0

:: ============================================================================
:: Functions
:: ============================================================================

:show_help
echo localsearch Installation Script for Windows
echo.
echo Usage: install.bat [OPTIONS]
echo.
echo Options:
echo   -InstallDir DIR     Installation directory (default: %%LOCALAPPDATA%%\Programs\localsearch)
echo   -GitHubRepo REPO    GitHub repository (default: nnanto/localsearch)
echo   -h, --help, /?      Show this help message
echo.
echo Examples:
echo   install.bat
echo   install.bat -InstallDir "C:\Tools\localsearch"
exit /b 0

:install_localsearch
set "ARCHIVE_NAME=localsearch-windows-x86_64.zip"
set "DOWNLOAD_URL=https://github.com/%GITHUB_REPO%/releases/latest/download/%ARCHIVE_NAME%"

echo [INFO] Download URL: %DOWNLOAD_URL%

:: Create temporary directory
set "TMP_DIR=%TEMP%\localsearch_%RANDOM%%RANDOM%"
mkdir "%TMP_DIR%" 2>nul

set "ARCHIVE_PATH=%TMP_DIR%\%ARCHIVE_NAME%"

:: Download archive
echo [INFO] Downloading localsearch...
curl -L -o "%ARCHIVE_PATH%" "%DOWNLOAD_URL%" --silent --show-error --fail
if errorlevel 1 (
    echo [ERROR] Failed to download localsearch
    call :cleanup "%TMP_DIR%"
    exit /b 1
)

:: Extract archive using PowerShell (compatible with older Windows versions)
echo [INFO] Extracting archive...
powershell -NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -Path '%ARCHIVE_PATH%' -DestinationPath '%TMP_DIR%' -Force"
if errorlevel 1 (
    echo [ERROR] Failed to extract archive
    call :cleanup "%TMP_DIR%"
    exit /b 1
)

:: Create install directory if it doesn't exist
if not exist "%INSTALL_DIR%" (
    echo [INFO] Creating installation directory: %INSTALL_DIR%
    mkdir "%INSTALL_DIR%"
)

:: Copy binary
set "BINARY_PATH=%TMP_DIR%\localsearch.exe"
set "TARGET_PATH=%INSTALL_DIR%\localsearch.exe"

if not exist "%BINARY_PATH%" (
    echo [ERROR] Binary not found in extracted archive
    call :cleanup "%TMP_DIR%"
    exit /b 1
)

echo [INFO] Installing to %INSTALL_DIR%...
copy /Y "%BINARY_PATH%" "%TARGET_PATH%" >nul
if errorlevel 1 (
    echo [ERROR] Failed to copy binary
    call :cleanup "%TMP_DIR%"
    exit /b 1
)

:: Add to PATH
call :add_to_path "%INSTALL_DIR%"

echo [INFO] localsearch installed successfully!
echo [INFO] Try running: localsearch --help
echo [INFO] You may need to restart your terminal for PATH changes to take effect.

:: Cleanup
call :cleanup "%TMP_DIR%"
exit /b 0

:add_to_path
set "DIR_TO_ADD=%~1"

:: Check if directory is already in PATH
echo %PATH% | find /i "%DIR_TO_ADD%" >nul
if not errorlevel 1 (
    echo [INFO] %DIR_TO_ADD% is already in PATH.
    exit /b 0
)

echo [INFO] Adding %DIR_TO_ADD% to PATH...

:: Get current user PATH
for /f "skip=2 tokens=3*" %%A in ('reg query "HKCU\Environment" /v PATH 2^>nul') do set "USER_PATH=%%A %%B"

:: Remove trailing space if exists
set "USER_PATH=!USER_PATH:~0,-1!"

:: Add new directory to PATH
if "!USER_PATH!"=="" (
    setx PATH "%DIR_TO_ADD%" >nul
) else (
    setx PATH "!USER_PATH!;%DIR_TO_ADD!" >nul
)

if errorlevel 1 (
    echo [WARN] Failed to add to PATH automatically. Please add manually: %DIR_TO_ADD%
    exit /b 1
)

:: Update PATH for current session
set "PATH=%PATH%;%DIR_TO_ADD%"

echo [INFO] Added to PATH. Please restart your terminal for changes to take effect.
exit /b 0

:cleanup
set "CLEANUP_DIR=%~1"
if exist "%CLEANUP_DIR%" (
    rd /s /q "%CLEANUP_DIR%" 2>nul
)
exit /b 0
 
