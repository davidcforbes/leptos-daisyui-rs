# test-visual.ps1 — orchestrate the PixelProof visual + reactivity smoke
# suites (ldui-49w.1). Invoked by `cargo make test-visual`.
#
#   1. npm install in demo/ (idempotent; Trunk's tailwind pre-build hook
#      needs node_modules — a fresh worktree won't have it)
#   2. start `trunk serve` in demo/ (background) unless port 3010 is already
#      serving (in which case reuse it and leave it running afterwards)
#   3. wait for http://127.0.0.1:3010/ to respond (first wasm build can take
#      minutes — generous timeout)
#   4. run the #[ignore]d suites
#   5. kill the trunk process tree (only if this script started it)
#
# Baseline refresh: VISUAL_TEST_MODE=capture flows straight through, e.g.
#   $env:VISUAL_TEST_MODE="capture"; cargo make test-visual

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$demoDir = Join-Path $repoRoot 'demo'
$baseUrl = if ($env:VISUAL_TEST_BASE_URL) { $env:VISUAL_TEST_BASE_URL } else { 'http://127.0.0.1:3010' }

function Test-Server {
    param([string]$Url)
    try {
        $resp = Invoke-WebRequest -Uri $Url -Method Get -TimeoutSec 5 -UseBasicParsing
        return $resp.StatusCode -eq 200
    } catch {
        return $false
    }
}

function Test-StylesheetCurrent {
    param([string]$Root)
    $stampPath = Join-Path $Root '.ldui-css-stamp'
    $distDir = Join-Path $Root 'dist'
    if (-not (Test-Path -LiteralPath $stampPath) -or -not (Test-Path -LiteralPath $distDir)) {
        return $false
    }

    $recorded = (Get-Content -LiteralPath $stampPath -Raw).Trim()
    $parts = $recorded -split '\s+', 2
    if ($parts.Count -ne 2 -or $parts[0] -ne 'ok') {
        return $false
    }

    $expected = "ldui-css-stamp-$($parts[1])"
    foreach ($stylesheet in Get-ChildItem -LiteralPath $distDir -Filter '*.css' -File) {
        if (Select-String -LiteralPath $stylesheet.FullName -SimpleMatch $expected -Quiet) {
            return $true
        }
    }
    return $false
}

$startedServer = $false
$trunkProc = $null

try {
    if (Test-Server $baseUrl) {
        Write-Host "test-visual: reusing already-running demo server at $baseUrl"
    } else {
        if (-not (Test-Path (Join-Path $demoDir 'node_modules'))) {
            Write-Host 'test-visual: npm install in demo/ (tailwind pre-build hook)'
            Push-Location $demoDir
            try { npm install; if ($LASTEXITCODE -ne 0) { throw "npm install failed ($LASTEXITCODE)" } }
            finally { Pop-Location }
        }

        # Mint a new stylesheet stamp before Trunk starts. This guarantees the
        # old distribution cannot satisfy readiness while Trunk is still
        # compiling and serving the previous dist/ directory.
        Write-Host 'test-visual: building and stamping demo stylesheet'
        Push-Location $demoDir
        try {
            node build-css.mjs
            if ($LASTEXITCODE -ne 0) { throw "stylesheet build failed ($LASTEXITCODE)" }
        } finally {
            Pop-Location
        }

        Write-Host 'test-visual: starting `trunk serve` in demo/ (background)'
        # Trunk 0.21 treats the conventional NO_COLOR=1 value as its boolean
        # --no-color option and rejects `1`. Remove it only while spawning the
        # child, matching xtask's browser runner, then restore this process.
        $savedNoColor = $env:NO_COLOR
        try {
            Remove-Item Env:NO_COLOR -ErrorAction SilentlyContinue
            $trunkArgs = @(
                'serve', '--address', '127.0.0.1', '--port', '3010',
                '--no-autoreload=true', '--open=false', '--color', 'never',
                '--release=true'
            )
            $trunkProc = Start-Process -FilePath 'trunk' -ArgumentList $trunkArgs `
                -WorkingDirectory $demoDir -PassThru -WindowStyle Hidden
        } finally {
            if ($null -eq $savedNoColor) {
                Remove-Item Env:NO_COLOR -ErrorAction SilentlyContinue
            } else {
                $env:NO_COLOR = $savedNoColor
            }
        }
        $startedServer = $true

        # First build of the wasm app can take several minutes.
        $deadline = (Get-Date).AddSeconds(900)
        Write-Host "test-visual: waiting for $baseUrl (up to 15 min for the first wasm build)"
        while (-not ((Test-Server $baseUrl) -and (Test-StylesheetCurrent $demoDir))) {
            if ($trunkProc.HasExited) {
                throw "trunk serve exited early with code $($trunkProc.ExitCode)"
            }
            if ((Get-Date) -gt $deadline) {
                throw "demo server did not come up at $baseUrl within 15 minutes"
            }
            Start-Sleep -Seconds 3
        }
        Write-Host 'test-visual: demo server assets are current'
    }

    $mode = if ($env:VISUAL_TEST_MODE) { $env:VISUAL_TEST_MODE } else { 'compare' }
    Write-Host "test-visual: running suites (VISUAL_TEST_MODE=$mode)"
    Push-Location $repoRoot
    try {
        # --test-threads=1: each test launches its own headless Chrome that
        # loads the ~60 MB dev wasm; parallel instances starve each other past
        # the mount-wait budget (observed: 2-7 mount timeouts at default
        # parallelism). Serial matches the desktop suite convention.
        cargo test --test visual_smoke --test reactivity_smoke -- --ignored --test-threads=1
        $testExit = $LASTEXITCODE
    } finally {
        Pop-Location
    }

    if ($testExit -ne 0) { throw "visual/reactivity suites failed (exit $testExit)" }
    Write-Host 'test-visual: PASS'
} finally {
    if ($startedServer -and $trunkProc -and -not $trunkProc.HasExited) {
        Write-Host "test-visual: stopping trunk serve (pid $($trunkProc.Id))"
        # /T kills the whole tree (trunk spawns cargo/wasm-bindgen children).
        & taskkill /PID $trunkProc.Id /T /F | Out-Null
    }
}
