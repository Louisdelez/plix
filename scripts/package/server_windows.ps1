# Package Plix headless server for Windows
#
# Creates a distributable zip archive containing:
# - plix-server-headless.exe binary
# - build_info.json
# - configs/examples/ directory
# - docs/README.md
# - run_server.ps1 launcher script
#
# Usage:
#   .\server_windows.ps1 -BinaryPath <path> -Version <semver> -OutputDir <path>
#
# Exit codes:
#   0 - Success
#   1 - Missing required argument
#   2 - Binary not found
#   3 - Config examples not found
#   4 - Archive creation failed

param(
    [Parameter(Mandatory=$true)]
    [string]$BinaryPath,

    [Parameter(Mandatory=$true)]
    [string]$Version,

    [Parameter(Mandatory=$true)]
    [string]$OutputDir
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)

# Validate binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found: $BinaryPath"
    exit 2
}

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Bundle naming
$Platform = "win64"
$BundleName = "plix-server-headless-$Platform-$Version"
$BundleDir = Join-Path $OutputDir $BundleName
$ArchivePath = Join-Path $OutputDir "$BundleName.zip"

Write-Host "Packaging plix-server-headless for Windows..."
Write-Host "  Version: $Version"
Write-Host "  Binary: $BinaryPath"
Write-Host "  Output: $ArchivePath"

# Clean up any existing bundle directory
if (Test-Path $BundleDir) {
    Remove-Item -Recurse -Force $BundleDir
}
New-Item -ItemType Directory -Force -Path $BundleDir | Out-Null

# Copy binary
Write-Host "Copying binary..."
Copy-Item $BinaryPath (Join-Path $BundleDir "plix-server-headless.exe")

# Copy example configs
Write-Host "Copying example configs..."
$ConfigDir = Join-Path $BundleDir "configs\examples"
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null

$SourceConfigDir = Join-Path $ProjectRoot "deploy\configs\examples"
if (Test-Path $SourceConfigDir) {
    Get-ChildItem -Path $SourceConfigDir -Filter "*.toml" | Copy-Item -Destination $ConfigDir
} else {
    # Create minimal example config
    @"
# Plix Server Configuration

[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "ffa_small"

[server]
name = "My Plix Server"
motd = "Welcome!"

[logging]
level = "info"

[persistence]
autosave_interval_secs = 300
shutdown_timeout_secs = 5
"@ | Out-File -Encoding UTF8 (Join-Path $ConfigDir "server.toml")
}

# Generate build_info.json
Write-Host "Generating build_info.json..."
$BuildInfoPath = Join-Path $BundleDir "build_info.json"

try {
    $CommitSha = git rev-parse HEAD 2>$null
    $CommitShort = git rev-parse --short HEAD 2>$null
    $Branch = git rev-parse --abbrev-ref HEAD 2>$null
    $IsDirty = (git diff --quiet 2>$null; if ($LASTEXITCODE -ne 0) { "true" } else { "false" })
} catch {
    $CommitSha = "unknown"
    $CommitShort = "unknown"
    $Branch = "unknown"
    $IsDirty = "false"
}

try {
    $RustVersionOutput = rustc --version 2>$null
    $RustVersionMatch = [regex]::Match($RustVersionOutput, '\d+\.\d+\.\d+')
    $RustVersion = if ($RustVersionMatch.Success) { $RustVersionMatch.Value } else { "unknown" }
} catch {
    $RustVersion = "unknown"
}

$BuildInfo = @{
    version = $Version
    commit_sha = $CommitSha
    commit_sha_short = $CommitShort
    build_timestamp = (Get-Date -Format "yyyy-MM-ddTHH:mm:ssZ")
    build_date = (Get-Date -Format "yyyy-MM-dd")
    target_triple = "x86_64-pc-windows-msvc"
    rust_version = $RustVersion
    branch = $Branch
    is_dirty = ($IsDirty -eq "true")
}

$BuildInfo | ConvertTo-Json | Out-File -Encoding UTF8 $BuildInfoPath

# Create launcher script
Write-Host "Creating launcher script..."
@'
# Plix Server Launcher
#
# Usage: .\run_server.ps1 [config_file]
#
# If no config file is specified, uses configs\examples\server.toml

param(
    [string]$ConfigFile
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Binary = Join-Path $ScriptDir "plix-server-headless.exe"

if (-not (Test-Path $Binary)) {
    Write-Error "Binary not found: $Binary"
    exit 1
}

if (-not $ConfigFile) {
    $ConfigFile = Join-Path $ScriptDir "configs\examples\server.toml"
}

if (Test-Path $ConfigFile) {
    & $Binary --config $ConfigFile
} else {
    Write-Warning "Config not found: $ConfigFile"
    Write-Host "Using default settings..."
    & $Binary
}
'@ | Out-File -Encoding UTF8 (Join-Path $BundleDir "run_server.ps1")

# Create docs directory with README
Write-Host "Creating documentation..."
$DocsDir = Join-Path $BundleDir "docs"
New-Item -ItemType Directory -Force -Path $DocsDir | Out-Null

@"
# Plix Headless Server

Version: $Version
Platform: Windows x64

## Quick Start

1. Edit the configuration file:
   ``````powershell
   Copy-Item configs\examples\server.toml .\server.toml
   notepad server.toml
   ``````

2. Start the server:
   ``````powershell
   .\run_server.ps1 server.toml
   # or directly:
   .\plix-server-headless.exe --config server.toml
   ``````

3. Stop the server:
   - Press Ctrl+C for graceful shutdown
   - Server will force-exit after shutdown timeout (default: 5s)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 64 | Port bind failed |
| 65 | Asset load failed |
| 66 | Persistence error |
| 67 | Network error |
| 68 | Shutdown timeout |

## Command Line Options

``````
.\plix-server-headless.exe --help
``````

## Configuration

See ``configs\examples\server.toml`` for all configuration options.

## Windows Service

To run as a Windows Service, use tools like NSSM (Non-Sucking Service Manager):

``````powershell
nssm install PlixServer C:\path\to\plix-server-headless.exe --config C:\path\to\server.toml
nssm start PlixServer
``````
"@ | Out-File -Encoding UTF8 (Join-Path $DocsDir "README.md")

# Validate bundle before archiving
Write-Host "Validating bundle..."
if (-not (Test-Path (Join-Path $BundleDir "plix-server-headless.exe"))) {
    Write-Error "Binary missing from bundle"
    exit 4
}
if (-not (Test-Path (Join-Path $BundleDir "build_info.json"))) {
    Write-Error "build_info.json missing from bundle"
    exit 4
}

# Create archive
Write-Host "Creating archive..."
if (Test-Path $ArchivePath) {
    Remove-Item $ArchivePath
}
Compress-Archive -Path $BundleDir -DestinationPath $ArchivePath

# Generate checksum
Write-Host "Generating checksum..."
$Hash = Get-FileHash -Algorithm SHA256 $ArchivePath
"$($Hash.Hash.ToLower())  $(Split-Path -Leaf $ArchivePath)" | Out-File -Encoding ASCII "$ArchivePath.sha256"

# Clean up bundle directory
Remove-Item -Recurse -Force $BundleDir

Write-Host "Package created successfully!"
Write-Host "  Archive: $ArchivePath"
Write-Host "  Checksum: $ArchivePath.sha256"

# Output archive path
Write-Output $ArchivePath
