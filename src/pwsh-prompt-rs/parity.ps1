# Compares the C# reference binary and the Rust port byte-for-byte across
# scenarios. Datetime output is masked before comparison because the two
# binaries run milliseconds apart.
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8

$repo = Split-Path (Split-Path $PSScriptRoot)
$cs = Join-Path $repo 'artifacts\bin\pwsh-prompt\release\pwsh-prompt.exe'
$rs = Join-Path $repo 'src\pwsh-prompt-rs\target\release\pwsh-prompt.exe'
$outDir = Join-Path $env:TEMP 'pwsh-parity'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

# Clean environment of prompt vars
Remove-Item Env:PROMPT_GIT_DIR_CACHED, Env:PROMPT_GIT_BRANCH_CACHED, Env:PROMPT_PR_NUMBER_CACHED, Env:PROMPT_PR_STATE_CACHED, Env:PROMPT_GIT_DIR, Env:PROMPT_GIT_BRANCH, Env:PROMPT_GIT_HEAD, Env:PROMPT_GIT_CACHE_DIR, Env:PROMPT_PR_NUMBER, Env:PROMPT_PR_STATE -ErrorAction SilentlyContinue

$maskDatetime = { param($text) [regex]::Replace([regex]::Replace($text, '\d{4}-\d{2}-\d{2} \d{1,2}:\d{2} (AM|PM)', 'DATETIME'), '\[\d+\.\d+ms\]', '[-ms]') }

function Compare-Scenario {
    param([string]$Name, [string[]]$ScenarioArgs, [hashtable]$ScenarioEnv = @{})
    $csArgs = @('prompt') + $ScenarioArgs
    foreach ($side in 'cs', 'rs') {
        $exe = if ($side -eq 'cs') { $cs } else { $rs }
        $saved = @{}
        foreach ($k in $ScenarioEnv.Keys) {
            $saved[$k] = [Environment]::GetEnvironmentVariable($k)
            [Environment]::SetEnvironmentVariable($k, $ScenarioEnv[$k])
        }
        $file = Join-Path $outDir "$Name-$side.bin"
        & $exe @csArgs > $file
        $code = $LASTEXITCODE
        foreach ($k in $saved.Keys) {
            [Environment]::SetEnvironmentVariable($k, $saved[$k])
        }
        if ($code -ne 0) { Write-Output "  !! $side exited $code" }
    }
    $csText = [regex]::Escape(([IO.File]::ReadAllText((Join-Path $outDir "$Name-cs.bin"))))
    $csMasked = & $maskDatetime ([IO.File]::ReadAllText((Join-Path $outDir "$Name-cs.bin")))
    $rsMasked = & $maskDatetime ([IO.File]::ReadAllText((Join-Path $outDir "$Name-rs.bin")))
    if ($csMasked -ceq $rsMasked) {
        Write-Output ("PASS  {0}" -f $Name)
    } else {
        Write-Output ("FAIL  {0}" -f $Name)
        Write-Output ("  cs: {0}" -f ($csMasked -replace "`e", '<ESC>'))
        Write-Output ("  rs: {0}" -f ($rsMasked -replace "`e", '<ESC>'))
    }
}

$repoDir = $repo
$subDir = Join-Path $repo 'src\pwsh-prompt-rs'
$home1 = $env:USERPROFILE
$nonrepo = 'C:\Windows'
$fakeLong = Join-Path $home1 '.nuget\packages\microsoft.extensions.primitives\10.0.11\lib\net10.0'

Write-Output "== init =="
& $cs init > (Join-Path $outDir 'init-cs.bin')
& $rs init > (Join-Path $outDir 'init-rs.bin')
$csInit = [IO.File]::ReadAllText((Join-Path $outDir 'init-cs.bin')).Replace($cs, '{{processName}}')
$rsInit = [IO.File]::ReadAllText((Join-Path $outDir 'init-rs.bin')).Replace($rs, '{{processName}}')
if ($csInit -ceq $rsInit) { Write-Output 'PASS  init (path-normalized)' } else {
    Write-Output 'FAIL  init'
    $a = $csInit.ToCharArray(); $b = $rsInit.ToCharArray()
    for ($i = 0; $i -lt [Math]::Min($a.Length, $b.Length); $i++) {
        if ($a[$i] -cne $b[$i]) { Write-Output ("  first diff at char {0}: cs={1} rs={2}" -f $i, [int]$a[$i], [int]$b[$i]); break }
    }
    Write-Output ("  lengths cs={0} rs={1}" -f $csInit.Length, $rsInit.Length)
}

Write-Output "== version =="
$csV = (& $cs --version).Trim()
$rsV = (& $rs --version).Trim()
Write-Output ("  cs: {0}" -f $csV)
Write-Output ("  rs: {0}" -f $rsV)
Write-Output ("  {0}" -f ($(if ($csV -ceq $rsV) { 'PASS  version' } else { 'FAIL  version' })))

Write-Output "== usage =="
& $cs > (Join-Path $outDir 'usage-cs.bin') 2>&1
& $rs > (Join-Path $outDir 'usage-rs.bin') 2>&1
$csU = [IO.File]::ReadAllText((Join-Path $outDir 'usage-cs.bin'))
$rsU = [IO.File]::ReadAllText((Join-Path $outDir 'usage-rs.bin'))
Write-Output ("  {0}" -f ($(if ($csU -ceq $rsU) { 'PASS  usage' } else { "FAIL  usage cs=$csU rs=$rsU" })))

Write-Output "== unknown verb prints usage =="
& $cs frobnicate > (Join-Path $outDir 'silent-cs.bin') 2>&1
& $rs frobnicate > (Join-Path $outDir 'silent-rs.bin') 2>&1
$csS = [IO.File]::ReadAllText((Join-Path $outDir 'silent-cs.bin'))
$rsS = [IO.File]::ReadAllText((Join-Path $outDir 'silent-rs.bin'))
Write-Output ("  {0}" -f ($(if ($csS -ceq $rsS) { 'PASS  silent' } else { "FAIL  silent cs='$csS' rs='$rsS'" })))

Write-Output "== prompt renders =="
$base = @('--terminal-width=120', '--last-command-state=true', '--last-command-exit-code=0', '--last-command-duration=0')
Compare-Scenario 'repo-root'     (@($base) + @("--current-directory=$repoDir"))
Compare-Scenario 'repo-sub'      (@($base) + @("--current-directory=$subDir"))
Compare-Scenario 'nonrepo'       (@($base) + @("--current-directory=$nonrepo"))
Compare-Scenario 'home'          (@($base) + @("--current-directory=$home1"))
Compare-Scenario 'home-truncated' (@('--terminal-width=60', '--last-command-state=true') + @("--current-directory=$home1"))
Compare-Scenario 'debug'          (@($base) + @("--current-directory=$repoDir")) @{ DEBUG_PROMPT = '1' }
Compare-Scenario 'debug-failed'   (@('--terminal-width=120', '--last-command-state=false', '--last-command-exit-code=130', '--last-command-duration=4500') + @("--current-directory=$repoDir")) @{ DEBUG_PROMPT = '1' }
Compare-Scenario 'simple-debug'   (@('--simple', '--terminal-width=120') + @("--current-directory=$repoDir")) @{ DEBUG_PROMPT = '1' }
Compare-Scenario 'narrow-long'   (@('--terminal-width=60', '--last-command-state=true') + @("--current-directory=$fakeLong"))
Compare-Scenario 'failed-cmd'    (@('--terminal-width=120', '--last-command-state=false', '--last-command-exit-code=130', '--last-command-duration=4500') + @("--current-directory=$repoDir"))
Compare-Scenario 'dur-29'        (@('--terminal-width=120', '--last-command-duration=29') + @("--current-directory=$nonrepo"))
Compare-Scenario 'dur-1000'      (@('--terminal-width=120', '--last-command-duration=1000') + @("--current-directory=$nonrepo"))
Compare-Scenario 'dur-59999'     (@('--terminal-width=120', '--last-command-duration=59999') + @("--current-directory=$nonrepo"))
Compare-Scenario 'dur-60000'     (@('--terminal-width=120', '--last-command-duration=60000') + @("--current-directory=$nonrepo"))
Compare-Scenario 'dur-61440000'  (@('--terminal-width=160', '--last-command-duration=61440000') + @("--current-directory=$nonrepo"))
Compare-Scenario 'dur-123456789' (@('--terminal-width=160', '--last-command-duration=123456789') + @("--current-directory=$nonrepo"))
Compare-Scenario 'exit-neg'      (@('--terminal-width=120', '--last-command-state=false', '--last-command-exit-code=-3') + @("--current-directory=$repoDir"))
Compare-Scenario 'state-true-code-nonzero' (@('--terminal-width=120', '--last-command-state=true', '--last-command-exit-code=7') + @("--current-directory=$repoDir"))
Compare-Scenario 'simple'        (@('--simple', '--terminal-width=120') + @("--current-directory=$repoDir"))
Compare-Scenario 'simple-narrow' (@('--simple', '--terminal-width=40') + @("--current-directory=$subDir"))

Write-Output "== git config branch match =="
$matchRepo = Join-Path $outDir 'branch-match-repo'
New-Item -ItemType Directory -Force -Path (Join-Path $matchRepo '.git') | Out-Null
[IO.File]::WriteAllText((Join-Path $matchRepo '.git\HEAD'), "ref: refs/heads/feature`n")
[IO.File]::WriteAllText((Join-Path $matchRepo '.git\config'), "[core]`n`tbody = false`n[branch `"dev`"]`n`tmerge = refs/heads/feature`n")
Compare-Scenario 'branch-match' (@($base) + @("--current-directory=$matchRepo"))
Remove-Item -Recurse -Force $matchRepo

Write-Output "== env-cached PR renders =="
Compare-Scenario 'pr-open'   (@($base) + @("--current-directory=$repoDir")) @{ PROMPT_GIT_DIR_CACHED = "$repoDir\.git"; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '12'; PROMPT_PR_STATE_CACHED = 'open' }
Compare-Scenario 'pr-closed' (@($base) + @("--current-directory=$repoDir")) @{ PROMPT_GIT_DIR_CACHED = "$repoDir\.git"; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '34'; PROMPT_PR_STATE_CACHED = 'closed' }
Compare-Scenario 'pr-draft'  (@($base) + @("--current-directory=$repoDir")) @{ PROMPT_GIT_DIR_CACHED = "$repoDir\.git"; PROMPT_GIT_BRANCH_CACHED = 'main'; PROMPT_PR_NUMBER_CACHED = '56'; PROMPT_PR_STATE_CACHED = 'draft' }
Compare-Scenario 'git-dir-cache-only' (@($base) + @("--current-directory=$repoDir")) @{ PROMPT_GIT_DIR_CACHED = "$repoDir\.git"; PROMPT_GIT_BRANCH_CACHED = '' }

Write-Output "== non-filesystem provider =="
Compare-Scenario 'non-filesystem' (@('--terminal-width=120') + @("--current-directory=HKLM:\Software") + @('--current-directory-is-filesystem=false'))
