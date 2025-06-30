param(
  [string]$Version = "latest",
  [string]$InstallPath = "$env:USERPROFILE\bin",
  [switch]$AddToPath = $true
)

# Configuration
$GITHUB_USER = "builtbyjb"
$GITHUB_REPO = "tide"
$BINARY_NAME = "tide.exe"
$EXECUTABLE_NAME = "tide"

Write-Host "Installing $EXECUTABLE_NAME..." -ForegroundColor Green