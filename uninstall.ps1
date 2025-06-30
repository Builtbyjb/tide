param(
  [string]$InstallPath = "$env:USERPROFILE\bin",
  [switch]$Force = $false,
  [switch]$RemoveFromPath = $true
)

# Configuration 
$EXECUTABLE_NAME = "tide-windows" 
$APP_NAME = "tide"

Write-Host "Uninstalling $APP_NAME..." -ForegroundColor Red

try {
  $binaryPath = "$InstallPath\$EXECUTABLE_NAME.exe"
  $found = $false
    
  # Check if binary exists
  if (Test-Path $binaryPath) {
    $found = $true
    Write-Host "Found installation at: $binaryPath" -ForegroundColor Yellow
      
    # Confirm uninstall unless forced
    if (-not $Force) {
      $confirmation = Read-Host "Are you sure you want to uninstall $APP_NAME? (y/N)"
      if ($confirmation -notmatch '^[Yy]') {
        Write-Host "Uninstall cancelled." -ForegroundColor Yellow
        exit 0
      }
    }
      
    # Stop any running processes
    Write-Host "Checking for running processes..." -ForegroundColor Yellow
    $processes = Get-Process -Name $EXECUTABLE_NAME -ErrorAction SilentlyContinue
    if ($processes) {
      Write-Host "Found $($processes.Count) running process(es). Stopping..." -ForegroundColor Yellow
      $processes | Stop-Process -Force
      Start-Sleep -Seconds 2
    }
      
    # Remove the binary
    Write-Host "Removing binary..." -ForegroundColor Yellow
    Remove-Item $binaryPath -Force -ErrorAction Stop
    Write-Host "Binary removed successfully" -ForegroundColor Green
  }
    
  # Remove from PATH if requested
  if ($RemoveFromPath) {
    Write-Host "Removing from PATH..." -ForegroundColor Yellow
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
      
    if ($currentPath -like "*$InstallPath*") {
      # Remove the install path from PATH
      $newPath = ($currentPath -split ';' | Where-Object { $_ -ne $InstallPath }) -join ';'
      $newPath = $newPath.TrimEnd(';')
        
      [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
      Write-Host "Removed from PATH" -ForegroundColor Green
      Write-Host "Restart your terminal for PATH changes to take effect" -ForegroundColor Cyan
    } else {
      Write-Host "Not found in PATH" -ForegroundColor Green
    }
  }
    
  # Check if install directory is empty and remove it
  if (Test-Path $InstallPath) {
    $remainingFiles = Get-ChildItem $InstallPath -ErrorAction SilentlyContinue
    if (-not $remainingFiles) {
      Write-Host "Removing empty install directory..." -ForegroundColor Yellow
      Remove-Item $InstallPath -Force -ErrorAction SilentlyContinue
      Write-Host "Install directory removed" -ForegroundColor Green
    } else {
      Write-Host "Install directory contains other files, keeping it" -ForegroundColor Yellow
    }
  }
    
  # Clean up temporary files
  Write-Host "Cleaning up temporary files..." -ForegroundColor Yellow
  $tempFiles = Get-ChildItem "$env:TEMP" -Filter "*$EXECUTABLE_NAME*" -ErrorAction SilentlyContinue
  if ($tempFiles) {
    $tempFiles | Remove-Item -Force -ErrorAction SilentlyContinue
    Write-Host "Temporary files cleaned" -ForegroundColor Green
  }
    
  if ($found) {
    Write-Host "Uninstall completed successfully!" -ForegroundColor Green
    Write-Host "$APP_NAME has been removed from your system." -ForegroundColor Cyan
  } else {
    Write-Host "No installation found" -ForegroundColor Yellow
    Write-Host "Searched for: $binaryPath" -ForegroundColor Yellow
    Write-Host "The application may not be installed or was installed to a different location." -ForegroundColor Yellow
  }
    
} catch {
  Write-Error "Uninstall failed: $($_.Exception.Message)"
  Write-Host "Manual cleanup may be required:" -ForegroundColor Yellow
  Write-Host "  1. Delete: $binaryPath" -ForegroundColor Yellow
  Write-Host "  2. Remove '$InstallPath' from your PATH environment variable" -ForegroundColor Yellow
  Write-Host "  3. Delete directory: $InstallPath (if empty)" -ForegroundColor Yellow
  exit 1
}