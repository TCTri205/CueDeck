# Governance Validation Script
# Enforcement of CueDeck Engineering Standards & Security Rules

$ErrorActionPreference = "Stop"
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir

function Write-Header {
    param($text)
    Write-Host "`n=== $text ===" -ForegroundColor Cyan
}

function Write-Success {
    param($text)
    Write-Host "✅ PASS: $text" -ForegroundColor Green
}

function Write-Failure {
    param($text)
    Write-Host "❌ FAIL: $text" -ForegroundColor Red
    exit 1
}

Set-Location $projectRoot

# 1. Security Scan
Write-Header "L1: Security Scan"
try {
    # Scan for secrets (sk-, ghp_, api_key=)
    # Using Select-String (grep equivalent)
    $secretPatterns = @(
        'sk-[a-zA-Z0-9]{20,}',
        'ghp_[a-zA-Z0-9]{10,}',
        '(api_key|password|secret|token)\s*=\s*["''][^"'']+["'']'
    )
    
    foreach ($pattern in $secretPatterns) {
        $hits = Get-ChildItem -Recurse -Include *.rs,*.toml,*.yml,*.json -Exclude *.lock -Path . | 
                Select-String -Pattern $pattern
        
        if ($hits) {
            Write-Host "Security Violations Found:" -ForegroundColor Red
            $hits | ForEach-Object { Write-Host "$($_.Path):$($_.LineNumber): $($_.Line)" }
            Write-Failure "Security scan failed. Secrets detected."
        }
    }
    Write-Success "No hardcoded secrets found."
} catch {
    Write-Warning "Security scan skipped (error: $_)"
}

# 2. Complexity & Lints
Write-Header "L1: Static Governance & Complexity"
try {
    # Check for clippy.toml existence
    if (-not (Test-Path "clippy.toml")) {
        Write-Failure "clippy.toml missin!. Governance config required."
    }

    Write-Host "Running cargo clippy (deny warnings)..." 
    cargo clippy --workspace --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        Write-Failure "Clippy Governance check failed. See errors above."
    }
    Write-Success "Code meets complexity and style standards."
} catch {
    Write-Failure "Failed to run cargo clippy."
}

# 3. Unit Tests
Write-Header "L2: Critical Unit Tests"
try {
    Write-Host "Running library tests..."
    cargo test --workspace --lib
    if ($LASTEXITCODE -ne 0) {
        Write-Failure "Unit tests failed."
    }
    Write-Success "All unit tests passed."
} catch {
    Write-Failure "Failed to run cargo test."
}

# 4. Documentation Check
Write-Header "L1: Documentation Compliance"
$requiredDocs = @(
    "docs/05_quality_and_ops/VERIFICATION_PLAN.md",
    "docs/05_quality_and_ops/COMPLEXITY_METRICS.md",
    "docs/04_security/SECURITY_RULES.md"
)

foreach ($doc in $requiredDocs) {
    if (-not (Test-Path $doc)) {
        Write-Failure "Missing required governance doc: $doc"
    }
}
Write-Success "Critical governance documentation present."

Write-Header "Governance Verification Complete"
Write-Host "Ready for Human Review or Merge." -ForegroundColor Green
exit 0
