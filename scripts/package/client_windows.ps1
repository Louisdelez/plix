# Package Plix client for Windows
#
# Creates a distributable zip archive containing:
# - plix-client.exe binary
# - build_info.json
# - assets/ directory
# - cef/ directory (if CEF UI enabled)
#
# Usage:
#   .\client_windows.ps1 -BinaryPath <path> -Version <semver> -OutputDir <path> [-AssetsDir <path>] [-CefDir <path>]
#
# Exit codes:
#   0 - Success
#   1 - Missing required argument
#   2 - Binary not found
#   3 - Assets not found
#   4 - Archive creation failed

param(
    [Parameter(Mandatory=$true)]
    [string]$BinaryPath,

    [Parameter(Mandatory=$true)]
    [string]$Version,

    [Parameter(Mandatory=$true)]
    [string]$OutputDir,

    [string]$AssetsDir = ".\assets",

    [string]$CefDir = ""
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Validate binary exists
if (-not (Test-Path $BinaryPath)) {
    Write-Error "Binary not found: $BinaryPath"
    exit 2
}

# Validate assets exist
if (-not (Test-Path $AssetsDir)) {
    Write-Error "Assets directory not found: $AssetsDir"
    exit 3
}

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Bundle naming
$Platform = "win64"
$BundleName = "plix-client-$Platform-$Version"
$BundleDir = Join-Path $OutputDir $BundleName
$ArchivePath = Join-Path $OutputDir "$BundleName.zip"

Write-Host "Packaging plix-client for Windows..."
Write-Host "  Version: $Version"
Write-Host "  Binary: $BinaryPath"
Write-Host "  Assets: $AssetsDir"
Write-Host "  Output: $ArchivePath"

# Clean up any existing bundle directory
if (Test-Path $BundleDir) {
    Remove-Item -Recurse -Force $BundleDir
}
New-Item -ItemType Directory -Force -Path $BundleDir | Out-Null

# Copy binary
Write-Host "Copying binary..."
Copy-Item $BinaryPath (Join-Path $BundleDir "plix-client.exe")

# Copy assets
Write-Host "Copying assets..."
Copy-Item -Recurse $AssetsDir (Join-Path $BundleDir "assets")

# Generate build_info.json
Write-Host "Generating build_info.json..."
$BuildInfoPath = Join-Path $BundleDir "build_info.json"

# Try to get version from binary
try {
    $VersionOutput = & (Join-Path $BundleDir "plix-client.exe") --version 2>&1
    $VersionMatch = [regex]::Match($VersionOutput, '\d+\.\d+\.\d+')
    $BinaryVersion = if ($VersionMatch.Success) { $VersionMatch.Value } else { $Version }
} catch {
    $BinaryVersion = $Version
}

# Get git info if available
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

# Get Rust version if available
try {
    $RustVersion = (rustc --version 2>$null) -replace 'rustc ', ''
    $RustVersionMatch = [regex]::Match($RustVersion, '\d+\.\d+\.\d+')
    $RustVersion = if ($RustVersionMatch.Success) { $RustVersionMatch.Value } else { "unknown" }
} catch {
    $RustVersion = "unknown"
}

$BuildInfo = @{
    version = $BinaryVersion
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

# Copy CEF runtime if specified
if ($CefDir -and (Test-Path $CefDir)) {
    Write-Host "Copying CEF runtime..."
    $CefBundleDir = Join-Path $BundleDir "cef"
    New-Item -ItemType Directory -Force -Path $CefBundleDir | Out-Null

    # Copy essential CEF files for Windows
    $CefFiles = @(
        "libcef.dll",
        "chrome_elf.dll",
        "d3dcompiler_47.dll",
        "icudtl.dat",
        "resources.pak",
        "chrome_100_percent.pak",
        "chrome_200_percent.pak",
        "v8_context_snapshot.bin"
    )

    foreach ($File in $CefFiles) {
        $SourcePath = Join-Path $CefDir $File
        $ReleasePath = Join-Path $CefDir "Release\$File"

        if (Test-Path $SourcePath) {
            Copy-Item $SourcePath $CefBundleDir
        } elseif (Test-Path $ReleasePath) {
            Copy-Item $ReleasePath $CefBundleDir
        }
    }

    # Copy locales
    $LocalesDir = Join-Path $CefDir "locales"
    $ResourcesLocalesDir = Join-Path $CefDir "Resources\locales"

    if (Test-Path $LocalesDir) {
        Copy-Item -Recurse $LocalesDir $CefBundleDir
    } elseif (Test-Path $ResourcesLocalesDir) {
        Copy-Item -Recurse $ResourcesLocalesDir $CefBundleDir
    }

    # Copy subprocess helper
    $SubprocessPath = Join-Path $CefDir "plix-client-subprocess.exe"
    if (Test-Path $SubprocessPath) {
        Copy-Item $SubprocessPath $CefBundleDir
    }
}

# Create optional config directory
New-Item -ItemType Directory -Force -Path (Join-Path $BundleDir "config\defaults") | Out-Null

# Validate bundle before archiving
Write-Host "Validating bundle..."
if (-not (Test-Path (Join-Path $BundleDir "plix-client.exe"))) {
    Write-Error "Binary missing from bundle"
    exit 4
}
if (-not (Test-Path (Join-Path $BundleDir "build_info.json"))) {
    Write-Error "build_info.json missing from bundle"
    exit 4
}
if (-not (Test-Path (Join-Path $BundleDir "assets"))) {
    Write-Error "assets directory missing from bundle"
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
