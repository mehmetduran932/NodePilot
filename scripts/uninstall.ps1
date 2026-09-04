# NodePilot Uninstall Script for Windows
$ErrorActionPreference = "SilentlyContinue"

$localAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
$binDir = Join-Path $localAppData "NodePilot\bin"

if (Test-Path (Join-Path $binDir "nodepilot.exe")) {
    Write-Host "Disabling NodePilot shell integrations..." -ForegroundColor Cyan
    & (Join-Path $binDir "nodepilot.exe") integration disable
}

Get-Process -Name "nodepilot" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue

if (Test-Path $binDir) {
    Remove-Item -Path $binDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "[v] NodePilot executables and shell integrations removed." -ForegroundColor Green
