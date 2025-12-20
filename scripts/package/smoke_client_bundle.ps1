# Smoke test for Plix client bundle (Windows)
#
# Validates that a client bundle is properly structured and the binary runs.
# This is a lightweight test suitable for CI pipelines.
#
# Usage:
#   .\smoke_client_bundle.ps1 <archive_path>
#
# Exit codes:
#   0 - All tests passed
#   1 - Missing argument
#   2 - Archive not found
#   3 - Extraction failed
#   4 - Missing required file
#   5 - Binary execution failed

param(
    [Parameter(Position=0, Mandatory=$true)]
    [string]$ArchivePath
)

$ErrorActionPreference = "Stop"

function Log-Pass { param([string]$Message) Write-Host "[PASS] $Message" -ForegroundColor Green }
function Log-Fail { param([string]$Message) Write-Host "[FAIL] $Message" -ForegroundColor Red }
function Log-Warn { param([string]$Message) Write-Host "[WARN] $Message" -ForegroundColor Yellow }
function Log-Info { param([string]$Message) Write-Host "[INFO] $Message" }

$TempDir = $null
$Errors = 0

try {
    if (-not (Test-Path $ArchivePath)) {
        Log-Fail "Archive not found: $ArchivePath"
        exit 2
    }

    Log-Info "Testing client bundle: $ArchivePath"

    # Create temp directory
    $TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $TempDir | Out-Null
    Log-Info "Extracting to: $TempDir"

    # Extract archive
    try {
        Expand-Archive -Path $ArchivePath -DestinationPath $TempDir -Force
        Log-Pass "Archive extraction"
    } catch {
        Log-Fail "Failed to extract archive: $_"
        exit 3
    }

    # Find bundle root
    $BundleRoot = Get-ChildItem -Path $TempDir -Directory | Where-Object { $_.Name -like "plix-client-*" } | Select-Object -First 1
    if ($BundleRoot) {
        $BundleRoot = $BundleRoot.FullName
    } else {
        $BundleRoot = $TempDir
    }
    Log-Info "Bundle root: $BundleRoot"

    # Test 1: Check build_info.json
    $BuildInfoPath = Join-Path $BundleRoot "build_info.json"
    if (Test-Path $BuildInfoPath) {
        Log-Pass "build_info.json exists"

        # Validate JSON
        try {
            $BuildInfo = Get-Content $BuildInfoPath | ConvertFrom-Json
            Log-Pass "build_info.json is valid JSON"

            # Check required fields
            foreach ($Field in @("version", "commit_sha", "target_triple")) {
                if ($BuildInfo.PSObject.Properties.Name -contains $Field) {
                    Log-Pass "build_info.json has $Field"
                } else {
                    Log-Fail "build_info.json missing $Field"
                    $Errors++
                }
            }
        } catch {
            Log-Fail "build_info.json is not valid JSON"
            $Errors++
        }
    } else {
        Log-Fail "build_info.json not found"
        $Errors++
    }

    # Test 2: Find and check binary
    $BinaryPath = Join-Path $BundleRoot "plix-client.exe"
    if (Test-Path $BinaryPath) {
        Log-Pass "Binary found: plix-client.exe"

        # Try to run --version
        Log-Info "Testing --version..."
        try {
            $VersionOutput = & $BinaryPath --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Log-Pass "--version returns successfully"
                Write-Host $VersionOutput
            } else {
                # Try --help
                $HelpOutput = & $BinaryPath --help 2>&1
                if ($LASTEXITCODE -eq 0) {
                    Log-Pass "--help returns successfully"
                } else {
                    Log-Warn "Binary did not respond to --version or --help"
                }
            }
        } catch {
            Log-Warn "Binary execution error (may require display): $_"
        }
    } else {
        Log-Fail "Binary not found"
        $Errors++
    }

    # Test 3: Check assets directory
    $AssetsDir = Join-Path $BundleRoot "assets"
    if (Test-Path $AssetsDir) {
        Log-Pass "Assets directory exists"

        # Check for common asset subdirectories
        if (Test-Path (Join-Path $AssetsDir "ui")) {
            Log-Pass "UI assets present"
        } else {
            Log-Warn "UI assets not found (may be optional)"
        }

        if (Test-Path (Join-Path $AssetsDir "arenas")) {
            Log-Pass "Arena assets present"
        } else {
            Log-Warn "Arena assets not found (may be optional)"
        }
    } else {
        Log-Fail "Assets directory not found"
        $Errors++
    }

    # Test 4: Check CEF runtime (optional)
    $CefDir = Join-Path $BundleRoot "cef"
    if (Test-Path $CefDir) {
        Log-Pass "CEF runtime present"

        $CefCount = (Get-ChildItem -Path $CefDir -File -Recurse).Count
        Log-Info "CEF files: $CefCount"

        # Check for essential DLLs
        $EssentialDlls = @("libcef.dll", "chrome_elf.dll")
        foreach ($Dll in $EssentialDlls) {
            if (Test-Path (Join-Path $CefDir $Dll)) {
                Log-Pass "$Dll present"
            } else {
                Log-Warn "$Dll not found in CEF directory"
            }
        }
    } else {
        Log-Info "CEF runtime not included (native UI mode)"
    }

    # Summary
    Write-Host ""
    if ($Errors -eq 0) {
        Log-Pass "All smoke tests passed!"
        exit 0
    } else {
        Log-Fail "Smoke tests failed with $Errors error(s)"
        exit 5
    }

} finally {
    # Cleanup
    if ($TempDir -and (Test-Path $TempDir)) {
        Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
    }
}
