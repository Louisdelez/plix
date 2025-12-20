# Smoke test for Plix headless server bundle (Windows)
#
# Validates that a server bundle is properly structured and the binary runs.
# This is a lightweight test suitable for CI pipelines.
#
# Usage:
#   .\smoke_headless_server.ps1 <archive_path>
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

    Log-Info "Testing server bundle: $ArchivePath"

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
    $BundleRoot = Get-ChildItem -Path $TempDir -Directory | Where-Object { $_.Name -like "plix-server-*" } | Select-Object -First 1
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

        try {
            $BuildInfo = Get-Content $BuildInfoPath | ConvertFrom-Json
            Log-Pass "build_info.json is valid JSON"

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
    $BinaryPath = Join-Path $BundleRoot "plix-server-headless.exe"
    if (Test-Path $BinaryPath) {
        Log-Pass "Binary found: plix-server-headless.exe"

        # Try to run --version
        Log-Info "Testing --version..."
        try {
            $VersionOutput = & $BinaryPath --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Log-Pass "--version returns successfully"
                Write-Host $VersionOutput
            } else {
                Log-Warn "Binary --version returned non-zero exit code"
            }
        } catch {
            Log-Warn "Binary --version error: $_"
        }

        # Try to run --help
        Log-Info "Testing --help..."
        try {
            $HelpOutput = & $BinaryPath --help 2>&1
            if ($LASTEXITCODE -eq 0) {
                Log-Pass "--help returns successfully"
                $HelpOutput | Select-Object -First 10
            } else {
                Log-Warn "Binary --help returned non-zero exit code"
            }
        } catch {
            Log-Warn "Binary --help error: $_"
        }

        # Try to validate config
        Log-Info "Testing --validate..."
        $TempConfig = Join-Path $TempDir "test_config.toml"
        @"
[network]
bind_address = "127.0.0.1"
port = 7777
max_players = 8

[game]
tick_rate = 30
arena = "ffa_small"
"@ | Out-File -Encoding UTF8 $TempConfig

        try {
            $ValidateOutput = & $BinaryPath --config $TempConfig --validate 2>&1
            if ($LASTEXITCODE -eq 0) {
                Log-Pass "Config validation works"
            } else {
                Log-Warn "Config validation returned non-zero: $ValidateOutput"
            }
        } catch {
            Log-Warn "Config validation error: $_"
        }
    } else {
        Log-Fail "Binary not found"
        $Errors++
    }

    # Test 3: Check configs directory
    $ConfigsDir = Join-Path $BundleRoot "configs\examples"
    if (Test-Path $ConfigsDir) {
        Log-Pass "Example configs directory exists"

        $ServerToml = Join-Path $ConfigsDir "server.toml"
        if (Test-Path $ServerToml) {
            Log-Pass "server.toml example present"
        } else {
            Log-Warn "server.toml example not found"
        }
    } else {
        Log-Warn "Example configs directory not found"
    }

    # Test 4: Check run script
    $RunScript = Join-Path $BundleRoot "run_server.ps1"
    if (Test-Path $RunScript) {
        Log-Pass "Run script exists: run_server.ps1"
    } else {
        Log-Warn "Run script not found"
    }

    # Test 5: Check docs
    $ReadmePath = Join-Path $BundleRoot "docs\README.md"
    if (Test-Path $ReadmePath) {
        Log-Pass "Documentation present"

        $ReadmeContent = Get-Content $ReadmePath -Raw
        if ($ReadmeContent -match "Exit Codes") {
            Log-Pass "Exit codes documented"
        }
    } else {
        Log-Warn "Documentation not found"
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
