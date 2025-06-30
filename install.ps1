param(
  [string]$Version = "0.1.1",
  [string]$InstallPath = "$env:USERPROFILE\bin",
  [switch]$AddToPath = $true
)

# Configuration
$GITHUB_USER = "builtbyjb"
$GITHUB_REPO = "tide"
$BINARY_NAME = "tide.exe"
$EXECUTABLE_NAME = "tide"

Write-Host "Installing $EXECUTABLE_NAME..." -ForegroundColor Green

try {
  # Get release information
  if ($Version -eq "latest") {
    $releaseUrl = "https://api.github.com/repos/$GITHUB_USER/$GITHUB_REPO/releases/latest"
    Write-Host "Fetching latest release information..." -ForegroundColor Yellow
  } else {
    $releaseUrl = "https://api.github.com/repos/$GITHUB_USER/$GITHUB_REPO/releases/tags/$Version"
    Write-Host "Fetching release information for version $Version..." -ForegroundColor Yellow
  }

   
  $release = Invoke-RestMethod $releaseUrl -ErrorAction Stop
    
  # Find the binary asset
  $asset = $release.assets | Where-Object { $_.name -eq $BINARY_NAME }
    
  if (-not $asset) {
    Write-Error "Binary '$BINARY_NAME' not found in release assets!"
    Write-Host "Available assets:" -ForegroundColor Yellow
    $release.assets | ForEach-Object { Write-Host "  - $($_.name)" }
    exit 1
  }
    
  Write-Host "Found asset: $($asset.name) ($(($asset.size / 1MB).ToString('F1')) MB)" -ForegroundColor Green
    
  # Create install directory
  if (-not (Test-Path $InstallPath)) {
    Write-Host "Creating install directory: $InstallPath" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
  }
    
  # Download the binary
  $downloadUrl = $asset.browser_download_url
  $tempFile = "$env:TEMP\$BINARY_NAME"
  $finalPath = "$InstallPath\$EXECUTABLE_NAME.exe"
    
  Write-Host "Downloading from: $downloadUrl" -ForegroundColor Yellow
  Write-Host "Downloading to: $tempFile" -ForegroundColor Yellow
    
  # Download with progress
  $webClient = New-Object System.Net.WebClient
  $webClient.DownloadFile($downloadUrl, $tempFile)
    
  Write-Host "Download completed!" -ForegroundColor Green
    
  # Move to final location
  Write-Host "Installing to: $finalPath" -ForegroundColor Yellow
  Move-Item $tempFile $finalPath -Force
    
  # Make executable (set permissions)
  $acl = Get-Acl $finalPath
  $accessRule = New-Object System.Security.AccessControl.FileSystemAccessRule(
    $env:USERNAME, "FullControl", "Allow"
  )
  $acl.SetAccessRule($accessRule)
  Set-Acl $finalPath $acl
    
  Write-Host "Installation completed successfully!" -ForegroundColor Green
  Write-Host "Binary installed at: $finalPath" -ForegroundColor Cyan
    
  # Add to PATH if requested
  if ($AddToPath) {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentPath -notlike "*$InstallPath*") {
      Write-Host "Adding $InstallPath to user PATH..." -ForegroundColor Yellow
      [Environment]::SetEnvironmentVariable(
        "Path", 
        "$currentPath;$InstallPath", 
        "User"
      )
      Write-Host "Added to PATH! Restart your terminal to use '$EXECUTABLE_NAME' command." -ForegroundColor Green
    } else {
      Write-Host "$InstallPath is already in PATH" -ForegroundColor Green
    }
  }
    
  # Test installation
  Write-Host "`nTesting installation..." -ForegroundColor Yellow
  if (Test-Path $finalPath) {
    $fileInfo = Get-Item $finalPath
    Write-Host "File exists: $($fileInfo.FullName)" -ForegroundColor Green
    Write-Host "File size: $(($fileInfo.Length / 1MB).ToString('F1')) MB" -ForegroundColor Green
    Write-Host "Created: $($fileInfo.CreationTime)" -ForegroundColor Green
      
    # Try to get version info if available
    try {
      $versionInfo = & $finalPath --version 2>$null
      if ($versionInfo) {
        Write-Host "Version: $versionInfo" -ForegroundColor Green
      }
    } catch {
        # Version command might not be available, that's okay
    }
  }
    
  Write-Host "Installation successful!" -ForegroundColor Green
  Write-Host "Run '$EXECUTABLE_NAME --help' to get started (after restarting terminal if added to PATH)" -ForegroundColor Cyan
} catch {
  Write-Error "Installation failed: $($_.Exception.Message)"
  Write-Host "Please check:" -ForegroundColor Yellow
  Write-Host "  1. Repository exists: https://github.com/$GITHUB_USER/$GITHUB_REPO" -ForegroundColor Yellow
  Write-Host "  2. Release exists with binary named '$BINARY_NAME'" -ForegroundColor Yellow
  Write-Host "  3. You have internet connection" -ForegroundColor Yellow
  exit 1
}