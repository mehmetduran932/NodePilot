# NodePilot One-Line Terminal Installer for Windows
# Usage: irm https://raw.githubusercontent.com/mehmetduran932/NodePilot/master/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host @"
   _   __          __     ____  _ __      __ 
  / | / /___  ____/ /__  / __ \(_) /___  / /_
 /  |/ / __ \/ __  / _ \/ /_/ / / / __ \/ __/
/ /|  / /_/ / /_/ /  __/ ____/ / / /_/ / /_  
/_/ |_/\____/\__,_/\___/_/   /_/_/\____/\__/   
"@ -ForegroundColor Cyan

Write-Host "Installing NodePilot (Cross-Platform Project-Aware Node.js Manager)..." -ForegroundColor Green

$repo = "mehmetduran932/NodePilot"
$localAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
$installBinDir = Join-Path $localAppData "NodePilot\bin"

# Ensure target directory exists
New-Item -ItemType Directory -Force -Path $installBinDir | Out-Null

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "NodePilot_Install_$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Force -Path $tempDir | Out-Null

try {
    Write-Host "Resolving latest release from GitHub ($repo)..." -ForegroundColor Gray
    $apiUrl = "https://api.github.com/repos/$repo/releases/latest"
    $headers = @{ "User-Agent" = "NodePilot-Installer" }
    
    $release = Invoke-RestMethod -Uri $apiUrl -Headers $headers -UseBasicParsing
    $zipAsset = $release.assets | Where-Object { $_.name -like "*windows-x64*.zip" } | Select-Object -First 1

    if (-not $zipAsset) {
        throw "Could not find a valid Windows zip asset in the latest release ($($release.tag_name))."
    }

    $zipPath = Join-Path $tempDir "nodepilot-windows-x64.zip"
    Write-Host "Downloading $($zipAsset.name) ($($release.tag_name))..." -ForegroundColor Cyan
    Invoke-WebRequest -Uri $zipAsset.browser_download_url -OutFile $zipPath -Headers $headers -UseBasicParsing

    Write-Host "Extracting binaries..." -ForegroundColor Gray
    Expand-Archive -Path $zipPath -DestinationPath $tempDir -Force

    # Locate binaries in extracted archive
    $nodepilotExe = Get-ChildItem -Path $tempDir -Filter "nodepilot.exe" -Recurse | Select-Object -ExpandProperty FullName -First 1
    $shimExe = Get-ChildItem -Path $tempDir -Filter "nodepilot-shim.exe" -Recurse | Select-Object -ExpandProperty FullName -First 1

    if (-not $nodepilotExe -or -not (Test-Path $nodepilotExe)) {
        throw "nodepilot.exe was not found in the downloaded archive."
    }

    # Stop any running instances if necessary
    Get-Process -Name "nodepilot" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

    Copy-Item $nodepilotExe (Join-Path $installBinDir "nodepilot.exe") -Force
    if ($shimExe -and (Test-Path $shimExe)) {
        Copy-Item $shimExe (Join-Path $installBinDir "nodepilot-shim.exe") -Force
    }

    Write-Host "Configuring shell integration and tool shims..." -ForegroundColor Cyan
    & (Join-Path $installBinDir "nodepilot.exe") integration enable

    # Update current session PATH so the user can immediately use it
    if ($env:Path -notlike "*$installBinDir*") {
        $env:Path = "$installBinDir;$env:Path"
    }

    Write-Host ""
    Write-Host "[v] NodePilot successfully installed to: $installBinDir" -ForegroundColor Green
    Write-Host "You can now run 'nodepilot' from any terminal window." -ForegroundColor Yellow
    Write-Host "Try: nodepilot --help or nodepilot current" -ForegroundColor Gray
}
finally {
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
