# Plix Server Launcher (Windows)
#
# This script provides a convenient way to start the Plix headless server
# with proper environment setup and logging.
#
# Usage:
#   .\run_server.ps1 [config_file]
#   .\run_server.ps1 -Help
#
# If no config file is specified, looks for server.toml in common locations.

param(
    [Parameter(Position=0)]
    [string]$ConfigFile,

    [switch]$Help
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

if ($Help) {
    Write-Host @"
Plix Server Launcher

Usage: .\run_server.ps1 [config_file]

Arguments:
  config_file    Path to server.toml config file (optional)

Config file search order (if not specified):
  1. .\server.toml
  2. %APPDATA%\plix\server.toml
  3. Built-in defaults

Environment Variables:
  PLIX_CONFIG     Override config file path
  PLIX_LOG_LEVEL  Override log level (trace, debug, info, warn, error)
  PLIX_PORT       Override server port
"@
    exit 0
}

# Find binary
$Binary = $null
$Candidates = @(
    (Join-Path $ScriptDir "plix-server-headless.exe"),
    (Join-Path $ScriptDir "..\plix-server-headless.exe"),
    ".\plix-server-headless.exe"
)

foreach ($Candidate in $Candidates) {
    if (Test-Path $Candidate) {
        $Binary = $Candidate
        break
    }
}

if (-not $Binary) {
    Write-Error "plix-server-headless.exe binary not found"
    Write-Host "Please ensure the binary is in the same directory as this script"
    exit 1
}

# Find config file
$Config = if ($env:PLIX_CONFIG) { $env:PLIX_CONFIG } else { $ConfigFile }

if (-not $Config) {
    $ConfigCandidates = @(
        ".\server.toml",
        (Join-Path $env:APPDATA "plix\server.toml")
    )

    foreach ($Candidate in $ConfigCandidates) {
        if (Test-Path $Candidate) {
            $Config = $Candidate
            Write-Host "Using config: $Config"
            break
        }
    }
}

# Build command line arguments
$Args = @()

if ($Config -and (Test-Path $Config)) {
    $Args += "--config"
    $Args += $Config
} elseif ($Config) {
    Write-Warning "Config file not found: $Config"
    Write-Host "Starting with default settings..."
}

# Apply environment variable overrides
if ($env:PLIX_LOG_LEVEL) {
    $Args += "--log-level"
    $Args += $env:PLIX_LOG_LEVEL
}

if ($env:PLIX_PORT) {
    $Args += "--port"
    $Args += $env:PLIX_PORT
}

# Print startup info
Write-Host "Starting Plix Server..."
Write-Host "  Binary: $Binary"
Write-Host "  Config: $(if ($Config) { $Config } else { 'default' })"
Write-Host "  Args: $($Args -join ' ')"
Write-Host ""

# Execute the server
& $Binary @Args
