namespace Prompt;

internal static class Init
{
    public static string GetPowerShell(string processName) =>
$$"""

  # Create a new dynamic module so we don't pollute the global namespace with our functions and variables
  $null = New-Module lucas-prompt {

      # Track history ID to detect first prompt and empty prompts (Ctrl+C, Enter with no command)
      [long]$script:lastHistoryId = -1

      function Invoke-Native {
          param($Executable, $Arguments)

          if (Test-Path $PWD.ProviderPath)
          {
              $workingDirectory = $PWD.ProviderPath;
          } else {
              $workingDirectory = $UserProfile;
          }

          $startInfo = New-Object System.Diagnostics.ProcessStartInfo -ArgumentList $Executable -Property @{
              StandardOutputEncoding = [System.Text.Encoding]::UTF8;
              RedirectStandardOutput = $true;
              RedirectStandardError = $true;
              CreateNoWindow = $true;
              UseShellExecute = $false;
              WorkingDirectory = $workingDirectory;
          };

          # requires PowerShell 6+ (or 6.1+)
          foreach ($arg in $Arguments) {
              $startInfo.ArgumentList.Add($arg);
          }

          $process = [System.Diagnostics.Process]::Start($startInfo)

          # Read the output and error streams asynchronously
          # Avoids potential deadlocks when the child process fills one of the buffers
          # https://docs.microsoft.com/en-us/dotnet/api/system.diagnostics.process.standardoutput?view=net-6.0#remarks
          $stdout = $process.StandardOutput.ReadToEndAsync()
          $stderr = $process.StandardError.ReadToEndAsync()
          [System.Threading.Tasks.Task]::WaitAll(@($stdout, $stderr))

          # stderr isn't displayed with this style of invocation
          # Manually write it to console
          if ($stderr.Result.Trim() -ne '') {
              # Write-Error doesn't work here
              $host.ui.WriteErrorLine($stderr.Result)
          }

          $stdout.Result;
      }

      function global:Prompt {
          $origDollarQuestion = $global:?
          $origLastExitCode = $global:LASTEXITCODE

          # Shell integration escape sequences (Windows Terminal, iTerm2, etc.)
          # ESC ] <code> ; <data> ST
          $e = [char]27
          $st = "$e\"
          $prompt = ''

          # OSC 133;D — mark previous command as finished (with exit code)
          # Skip on first prompt (no command has run yet)
          $lastHistory = Get-History -Count 1
          if ($script:lastHistoryId -ne -1) {
              if ($lastHistory.Id -eq $script:lastHistoryId) {
                  # No new command was executed (e.g. Ctrl+C, empty Enter)
                  $prompt += "$e]133;D$st"
              } else {
                  $exitCode = if ($origDollarQuestion) { 0 } else { if ($origLastExitCode) { $origLastExitCode } else { 1 } }
                  $prompt += "$e]133;D;$exitCode$st"
              }
          }

          # OSC 133;A — mark prompt start
          $prompt += "$e]133;A$st"

          # OSC 9;9 — communicate current working directory (for new tab same directory)
          $prompt += "$e]9;9;`"$($PWD.ProviderPath)`"$st"

          # Discover git directory when working directory changes.
          # Done in PowerShell because child process environment changes (set by the C# binary)
          # do not propagate back to the parent PowerShell process.
          if ($env:PROMPT_GIT_CACHE_DIR -ne $PWD.Path) {
              $env:PROMPT_GIT_DIR = ''
              $searchPath = $PWD.Path
              while ($searchPath) {
                  $gitPath = [IO.Path]::Combine($searchPath, '.git')
                  if ([IO.Directory]::Exists($gitPath)) {
                      $env:PROMPT_GIT_DIR = $gitPath
                      break
                  }
                  if ([IO.File]::Exists($gitPath)) {
                      # Git worktree: .git is a file containing "gitdir: <path>"
                      $content = [IO.File]::ReadAllText($gitPath)
                      if ($content -match 'gitdir:\s*(.+)') {
                          $worktreeDir = $matches[1].Trim()
                          if (-not [IO.Path]::IsPathRooted($worktreeDir)) {
                              $worktreeDir = [IO.Path]::GetFullPath([IO.Path]::Combine($searchPath, $worktreeDir))
                          }
                          if ([IO.Directory]::Exists($worktreeDir)) {
                              $env:PROMPT_GIT_DIR = $worktreeDir
                          }
                      }
                      break
                  }
                  $parent = [IO.Path]::GetDirectoryName($searchPath)
                  if (!$parent -or $parent -eq $searchPath) { break }
                  $searchPath = $parent
              }
          }

          # Check if HEAD changed (detects branch switches, rebases, etc.)
          $headChanged = $false
          if ($env:PROMPT_GIT_CACHE_DIR -eq $PWD.Path -and $env:PROMPT_GIT_DIR) {
              $headFile = [IO.Path]::Combine($env:PROMPT_GIT_DIR, 'HEAD')
              if ([IO.File]::Exists($headFile)) {
                  $currentHead = [IO.File]::ReadAllText($headFile)
                  if ($currentHead -ne $env:PROMPT_GIT_HEAD) {
                      $headChanged = $true
                  }
              }
          }

          if ($env:PROMPT_GIT_CACHE_DIR -eq $PWD.Path -and -not $headChanged) {
              $env:PROMPT_GIT_DIR_CACHED = $env:PROMPT_GIT_DIR
              $env:PROMPT_GIT_BRANCH_CACHED = $env:PROMPT_GIT_BRANCH
              $env:PROMPT_PR_NUMBER_CACHED = $env:PROMPT_PR_NUMBER
          } else {
              # Pass PowerShell-discovered git dir through so C# skips its own walk.
              $env:PROMPT_GIT_DIR_CACHED = $env:PROMPT_GIT_DIR
              $env:PROMPT_GIT_BRANCH_CACHED = ''

              # Fetch PR number on branch change (skip if gh is not available)
              if ((Get-Command gh -ErrorAction SilentlyContinue)) {
                  $env:PROMPT_PR_NUMBER = (gh pr view --json number -q .number 2>$null)
              } else {
                  $env:PROMPT_PR_NUMBER = ''
              }
              $env:PROMPT_PR_NUMBER_CACHED = $env:PROMPT_PR_NUMBER
          }

          $invariant = [System.Globalization.CultureInfo]::InvariantCulture
          $durationMs = ([int](Get-History -Count 1).Duration.TotalMilliseconds).ToString($invariant)
          $arguments = @(
              "prompt",
              "--terminal-width=$($Host.UI.RawUI.WindowSize.Width)",
              "--current-directory=$($PWD.Path)",
              "--current-directory-is-filesystem=$($PWD.Provider.Name -eq 'FileSystem')",
              "--last-command-state=$origDollarQuestion",
              "--last-command-exit-code=$LASTEXITCODE",
              "--last-command-duration=$durationMs"
          )

          # Invoke <Prompt>
          $promptText = Invoke-Native -Executable '{{processName}}' -Arguments $arguments

          # Cache git info for next prompt
          $env:PROMPT_GIT_CACHE_DIR = $PWD.Path
          if ($env:PROMPT_GIT_DIR) {
              $headFile = [IO.Path]::Combine($env:PROMPT_GIT_DIR, 'HEAD')
              if ([IO.File]::Exists($headFile)) {
                  $headContent = [IO.File]::ReadAllText($headFile)
                  $env:PROMPT_GIT_HEAD = $headContent
                  # Parse branch name for C# branch cache optimization
                  if ($headContent -match 'ref: refs/heads/(.+)') {
                      $env:PROMPT_GIT_BRANCH = $matches[1].Trim()
                  } else {
                      $env:PROMPT_GIT_BRANCH = ''
                  }
              }
          }

          # notify PSReadLine of a multiline prompt
          Set-PSReadLineOption -ExtraPromptLineCount (($promptText | Measure-Object -Line).Lines - 1)

          # Return the full prompt with shell integration marks
          # OSC 133;B — mark end of prompt / start of command input
          $prompt + $promptText + "$e]133;B$st"

          # Track history ID for next prompt's 133;D logic
          $script:lastHistoryId = $lastHistory.Id

          # Propagate the original $LASTEXITCODE from before the prompt function was invoked.
          $global:LASTEXITCODE = $origLastExitCode

          # Propagate the original $? automatic variable value from before the prompt function was invoked.
          #
          # $? is a read-only or constant variable so we can't directly override it.
          # In order to propagate up its original boolean value we will take an action
          # which will produce the desired value.
          #
          # This has to be the very last thing that happens in the prompt function
          # since every PowerShell command sets the $? variable.
          if ($global:? -ne $origDollarQuestion) {
              if ($origDollarQuestion) {
                  # Simple command which will execute successfully and set $? = True without any other side affects.
                  1+1
              } else {
                  # Write-Error will set $? to False.
                  # ErrorAction Ignore will prevent the error from being added to the $Error collection.
                  Write-Error '' -ErrorAction 'Ignore'
              }
          }
      }

      # OSC 133;C — mark start of command output (emitted when user presses Enter)
      Set-PSReadLineKeyHandler -Key Enter -ScriptBlock {
          [Console]::Write("$([char]27)]133;C$([char]27)\")
          [Microsoft.PowerShell.PSConsoleReadLine]::AcceptLine()
      }
  }

  """;
}
