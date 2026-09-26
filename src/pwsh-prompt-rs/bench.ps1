# Compares end-to-end performance of the C# reference binary and the Rust
# port: process startup plus a full prompt render, timed per scenario.
#
# Requirements: both toolchains for a full comparison (cargo for the Rust
# binary, .NET SDK for the C# binary). If the C# binary cannot be produced,
# the script falls back to Rust-only statistics.
#
# Usage: pwsh -File src/pwsh-prompt-rs/bench.ps1 [-Runs 100] [-Warmup 20]
#                [-SkipBuild] [-RustOnly]
#Requires -Version 7.0

param(
    [int]$Runs = 100,
    [int]$Warmup = 20,
    [switch]$SkipBuild,
    [switch]$RustOnly
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8

$repo = Split-Path (Split-Path $PSScriptRoot)
$exeName = if ($IsWindows) { 'pwsh-prompt.exe' } else { 'pwsh-prompt' }
$rsExe = Join-Path $repo "src/pwsh-prompt-rs/target/release/$exeName"
$csExe = Join-Path $repo "artifacts/bin/pwsh-prompt/release/$exeName"

# All environment variables the binaries read; cleared between scenarios so
# a variable set for one scenario cannot leak into the next. Shells and agent
# processes often inherit live PROMPT_* values, which would silently turn
# every scenario into a cache hit.
$PromptEnvNames = @(
    'PROMPT_GIT_DIR_CACHED',
    'PROMPT_GIT_BRANCH_CACHED',
    'PROMPT_PR_NUMBER_CACHED',
    'PROMPT_PR_STATE_CACHED',
    'PROMPT_GIT_DIR',
    'PROMPT_GIT_BRANCH',
    'PROMPT_GIT_HEAD',
    'PROMPT_GIT_CACHE_DIR',
    'PROMPT_PR_NUMBER',
    'PROMPT_PR_STATE'
)
# DEBUG_PROMPT must stay unset during measurement; the debug listing would
# dominate the timings.
$PromptEnvNames += 'DEBUG_PROMPT'

function Clear-PromptEnv {
    foreach ($name in $PromptEnvNames) {
        Remove-Item "Env:$name" -ErrorAction SilentlyContinue
    }
}

Clear-PromptEnv

# Resolve (and if needed, build) the binaries.
if (-not (Test-Path $rsExe) -and -not $SkipBuild) {
    Write-Host "Building Rust binary..." -ForegroundColor Cyan
    cargo build --release --manifest-path (Join-Path $repo 'src/pwsh-prompt-rs/Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
}
if (-not (Test-Path $rsExe)) { throw "Rust binary not found at: $rsExe (build it with cargo build --release)" }

$csAvailable = $true
if (-not $RustOnly) {
    if (-not (Test-Path $csExe) -and -not $SkipBuild) {
        Write-Host "Building C# binary..." -ForegroundColor Cyan
        dotnet build (Join-Path $repo 'src/pwsh-prompt') -c Release
        if ($LASTEXITCODE -ne 0) {
            Write-Warning "dotnet build failed (exit $LASTEXITCODE); continuing with Rust-only statistics."
            $csAvailable = $false
        }
    }
    if (-not (Test-Path $csExe)) {
        Write-Warning "C# binary not found at: $csExe; continuing with Rust-only statistics."
        $csAvailable = $false
    }
} else {
    $csAvailable = $false
}

# One timed run: starts the binary with the given arguments and scenario
# environment (per-process, so the parent environment stays clean), waits for
# exit, and returns the elapsed milliseconds plus the captured output.
function Invoke-TimedRun {
    param(
        [string]$Exe,
        [string[]]$Args,
        [hashtable]$Env
    )

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = $Exe
    foreach ($a in $Args) { $psi.ArgumentList.Add($a) }
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true

    # Start from the (already cleaned) parent environment, apply the scenario
    # variables, and remove every other prompt variable so nothing leaks
    # between scenarios.
    foreach ($name in $PromptEnvNames) {
        if ($Env.ContainsKey($name)) {
            $psi.Environment[$name] = $Env[$name]
        } else {
            $null = $psi.Environment.Remove($name)
        }
    }

    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $proc = [System.Diagnostics.Process]::Start($psi)
    $stdoutTask = $proc.StandardOutput.ReadToEndAsync()
    $stderrTask = $proc.StandardError.ReadToEndAsync()
    $proc.WaitForExit()
    $sw.Stop()

    [pscustomobject]@{
        Ms       = $sw.Elapsed.TotalMilliseconds
        ExitCode = $proc.ExitCode
        StdOut   = $stdoutTask.Result
        StdErr   = $stderrTask.Result
    }
}

function Get-Stats {
    param([double[]]$Samples)

    $sorted = $Samples | Sort-Object
    $count = $sorted.Count
    $median = if ($count % 2 -eq 1) { $sorted[[int](($count - 1) / 2)] }
              else { ($sorted[$count / 2 - 1] + $sorted[$count / 2]) / 2 }
    $p95Index = [Math]::Min($count - 1, [int][Math]::Ceiling(0.95 * $count) - 1)

    [pscustomobject]@{
        Min    = $sorted[0]
        Mean   = ($Samples | Measure-Object -Average).Average
        Median = $median
        P95    = $sorted[$p95Index]
    }
}

function Format-Ms {
    param([double]$Ms)
    '{0,10:N2} ms' -f $Ms
}

# Scenario set: the same inputs the byte-parity harness uses, plus cold and
# warm git-discovery variants. Base arguments mirror a normal prompt call.
$baseArgs = @('--terminal-width=120', '--last-command-state=true', '--last-command-exit-code=0', '--last-command-duration=0')
$subDir = Join-Path $repo 'src/pwsh-prompt-rs'
$nonRepo = if ($IsWindows) { 'C:\Windows' } else { '/usr' }
$homeDir = $env:USERPROFILE; if (-not $homeDir) { $homeDir = $env:HOME }
$longPath = Join-Path $homeDir '.nuget/packages/microsoft.extensions.primitives/10.0.11/lib/net10.0'
$repoGitDir = Join-Path $repo '.git'

$warmCache = @{ PROMPT_GIT_DIR_CACHED = $repoGitDir; PROMPT_GIT_BRANCH_CACHED = 'main' }

$scenarios = @(
    @{ Name = 'repo-root';       Args = @($baseArgs) + @("--current-directory=$repo");   Env = $warmCache },
    @{ Name = 'repo-root-cold';  Args = @($baseArgs) + @("--current-directory=$repo");   Env = @{} },
    @{ Name = 'repo-sub';        Args = @($baseArgs) + @("--current-directory=$subDir"); Env = $warmCache },
    @{ Name = 'non-repo';        Args = @($baseArgs) + @("--current-directory=$nonRepo"); Env = @{} },
    @{ Name = 'home';            Args = @($baseArgs) + @("--current-directory=$homeDir"); Env = @{} },
    @{ Name = 'long-path-narrow'; Args = @('--terminal-width=60', '--last-command-state=true', '--last-command-exit-code=0', '--last-command-duration=0') + @("--current-directory=$longPath"); Env = @{} },
    @{ Name = 'simple';          Args = @('--simple', '--terminal-width=120') + @("--current-directory=$repo"); Env = $warmCache },
    @{ Name = 'pr-open';         Args = @($baseArgs) + @("--current-directory=$repo");   Env = @{ PROMPT_GIT_DIR_CACHED = $repoGitDir; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '12'; PROMPT_PR_STATE_CACHED = 'open' } },
    @{ Name = 'pr-closed';       Args = @($baseArgs) + @("--current-directory=$repo");   Env = @{ PROMPT_GIT_DIR_CACHED = $repoGitDir; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '34'; PROMPT_PR_STATE_CACHED = 'closed' } },
    @{ Name = 'pr-draft';        Args = @($baseArgs) + @("--current-directory=$repo");   Env = @{ PROMPT_GIT_DIR_CACHED = $repoGitDir; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '56'; PROMPT_PR_STATE_CACHED = 'draft' } }
)

Write-Output 'pwsh-prompt end-to-end benchmark (process startup + prompt render)'
Write-Output ("OS:            {0}" -f [System.Runtime.InteropServices.RuntimeInformation]::OSDescription)
Write-Output ("CPUs:          {0}" -f [System.Environment]::ProcessorCount)
Write-Output ("Runs/scenario: {0} timed, {1} warmup, interleaved" -f $Runs, $Warmup)
Write-Output ("Rust binary:   {0}" -f $rsExe)
if ($csAvailable) { Write-Output ("C# binary:     {0}" -f $csExe) } else { Write-Output "C# binary:     (not available, Rust-only mode)" }
Write-Output ''

# Consistency precheck: the implementations must still render identical
# output (modulo the datetime), otherwise the comparison is meaningless.
$maskDatetime = { param($text) [regex]::Replace($text, '\d{4}-\d{2}-\d{2} \d{1,2}:\d{2} (AM|PM)', 'DATETIME') }
$precheck = $scenarios | Where-Object { $_.Name -eq 'repo-root' }
$csPre = Invoke-TimedRun -Exe $csExe -Args (@('prompt') + $precheck.Args) -Env $precheck.Env
$rsPre = Invoke-TimedRun -Exe $rsExe -Args (@('prompt') + $precheck.Args) -Env $precheck.Env
$csMasked = & $maskDatetime $csPre.StdOut
$rsMasked = & $maskDatetime $rsPre.StdOut
if ($csMasked -cne $rsMasked) {
    Write-Warning "Output mismatch between the binaries (datetime masked); compare with parity.ps1 before trusting these numbers."
}

# Warmup (unmeasured), then interleaved timed runs so thermal and cache
# effects hit both binaries equally.
foreach ($scenario in $scenarios) {
    $args = @('prompt') + $scenario.Args
    for ($i = 0; $i -lt $Warmup; $i++) {
        $null = Invoke-TimedRun -Exe $rsExe -Args $args -Env $scenario.Env
        if ($csAvailable) { $null = Invoke-TimedRun -Exe $csExe -Args $args -Env $scenario.Env }
    }
}

$results = foreach ($scenario in $scenarios) {
    $args = @('prompt') + $scenario.Args
    $csSamples = [System.Collections.Generic.List[double]]::new()
    $rsSamples = [System.Collections.Generic.List[double]]::new()
    $warnedExit = @{}

    for ($i = 0; $i -lt $Runs; $i++) {
        if ($csAvailable) {
            $run = Invoke-TimedRun -Exe $csExe -Args $args -Env $scenario.Env
            if ($run.ExitCode -ne 0 -and -not $warnedExit['cs']) {
                Write-Warning ("C# binary exited {0} for scenario {1}; stderr: {2}" -f $run.ExitCode, $scenario.Name, $run.StdErr)
                $warnedExit['cs'] = $true
            }
            $csSamples.Add($run.Ms)
        }
        $run = Invoke-TimedRun -Exe $rsExe -Args $args -Env $scenario.Env
        if ($run.ExitCode -ne 0 -and -not $warnedExit['rs']) {
            Write-Warning ("Rust binary exited {0} for scenario {1}; stderr: {2}" -f $run.ExitCode, $scenario.Name, $run.StdErr)
            $warnedExit['rs'] = $true
        }
        $rsSamples.Add($run.Ms)
    }

    $rsStats = Get-Stats $rsSamples.ToArray()
    $row = [ordered]@{
        Scenario   = $scenario.Name
        RustMedian = $rsStats.Median
        RustMin    = $rsStats.Min
        RustMean   = $rsStats.Mean
        RustP95    = $rsStats.P95
    }
    if ($csAvailable) {
        $csStats = Get-Stats $csSamples.ToArray()
        $row.CsMedian = $csStats.Median
        $row.CsMin = $csStats.Min
        $row.CsMean = $csStats.Mean
        $row.CsP95 = $csStats.P95
        $row.Ratio = $rsStats.Median / $csStats.Median
    }
    [pscustomobject]$row
}

# Table
$nameWidth = ($results.Scenario | Measure-Object -Maximum -Property Length).Maximum + 2
if ($csAvailable) {
    $header = '{0}{1}{2}{3}{4}' -f `
        'scenario'.PadRight($nameWidth),
        (Format-Ms 0).Replace('0.00 ms', 'c# median').PadLeft(14),
        (Format-Ms 0).Replace('0.00 ms', 'rust median').PadLeft(15),
        (Format-Ms 0).Replace('0.00 ms', 'rust min').PadLeft(14),
        'ratio'.PadLeft(9)
} else {
    $header = '{0}{1}{2}' -f `
        'scenario'.PadRight($nameWidth),
        (Format-Ms 0).Replace('0.00 ms', 'rust median').PadLeft(15),
        (Format-Ms 0).Replace('0.00 ms', 'rust min').PadLeft(14)
}
Write-Output $header
foreach ($row in $results) {
    $line = $row.Scenario.PadRight($nameWidth)
    if ($csAvailable) {
        $line += '{0}{1}' -f (Format-Ms $row.CsMedian), (Format-Ms $row.RustMedian)
        $line += (Format-Ms $row.RustMin)
        $line += ('{0,9:N2}' -f $row.Ratio)
    } else {
        $line += (Format-Ms $row.RustMedian)
        $line += (Format-Ms $row.RustMin)
    }
    Write-Output $line
}
