#Requires -Version 5.0

param(
    [Parameter()]
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\local-search",
    
    [Parameter()]
    [string]$GitHubRepo = "nnanto/local_search",
    
    [Parameter()]
    [switch]$Help
)

# Color functions
function Write-Status {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Show-Help {
    Write-Host "local-search Installation Script for Windows"
    Write-Host ""
    Write-Host "Usage: .\install.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -InstallDir DIR     Installation directory (default: $env:LOCALAPPDATA\Programs\local-search)"
    Write-Host "  -GitHubRepo REPO    GitHub repository (default: nnanto/local_search)"
    Write-Host "  -Help               Show this help message"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1"
    Write-Host "  .\install.ps1 -InstallDir 'C:\Tools\local-search'"
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Add-ToPath {
    param([string]$Directory)
    
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -split ';' -notcontains $Directory) {
        Write-Status "Adding $Directory to PATH..."
        [Environment]::SetEnvironmentVariable("Path", "$currentPath;$Directory", "User")
        $env:Path = "$env:Path;$Directory"
        Write-Status "Added to PATH. You may need to restart your terminal."
    } else {
        Write-Status "$Directory is already in PATH."
    }
}

function Install-LocalSearch {
    $archiveName = "local-search-windows-x86_64.zip"
    $downloadUrl = "https://github.com/$GitHubRepo/releases/latest/download/$archiveName"
    
    Write-Status "Download URL: $downloadUrl"
    
    # Create temporary directory
    $tmpDir = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmpDir -Force | Out-Null
    
    try {
        # Download archive
        $archivePath = Join-Path $tmpDir $archiveName
        Write-Status "Downloading local-search..."
        
        try {
            Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
        } catch {
            Write-Error "Failed to download local-search: $_"
            return $false
        }
        
        # Extract archive
        Write-Status "Extracting archive..."
        try {
            Expand-Archive -Path $archivePath -DestinationPath $tmpDir -Force
        } catch {
            Write-Error "Failed to extract archive: $_"
            return $false
        }
        
        # Create install directory
        if (!(Test-Path $InstallDir)) {
            Write-Status "Creating installation directory: $InstallDir"
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        
        # Copy binary
        $binaryPath = Join-Path $tmpDir "local-search.exe"
        $targetPath = Join-Path $InstallDir "local-search.exe"
        
        if (!(Test-Path $binaryPath)) {
            Write-Error "Binary not found in extracted archive"
            return $false
        }
        
        Write-Status "Installing to $InstallDir..."
        Copy-Item -Path $binaryPath -Destination $targetPath -Force
        
        # Add to PATH
        Add-ToPath -Directory $InstallDir
        
        Write-Status "local-search installed successfully!"
        Write-Status "Try running: local-search --help"
        
        return $true
        
    } finally {
        # Cleanup
        if (Test-Path $tmpDir) {
            Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

# Main script logic
if ($Help) {
    Show-Help
    exit 0
}

Write-Status "Installing local-search CLI tool..."
Write-Status "Installation directory: $InstallDir"
Write-Status "GitHub repository: $GitHubRepo"

if (Install-LocalSearch) {
    Write-Status "Installation completed successfully!"
} else {
    Write-Error "Installation failed!"
    exit 1
}