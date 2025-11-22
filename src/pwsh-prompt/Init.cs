namespace Prompt;

internal static class Init
{
    public static string GetPowerShell(string processName) =>
$$"""
  
  # Create a new dynamic module so we don't pollute the global namespace with our functions and variables
  $null = New-Module lucas-prompt {
  
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

          # Pass cached git info via environment variables if still in same directory and HEAD unchanged
          $headChanged = $false
          if ($env:PROMPT_GIT_CACHE_DIR -eq $PWD.Path -and $env:PROMPT_GIT_DIR) {
              $headFile = Join-Path $env:PROMPT_GIT_DIR "HEAD"
              if (Test-Path $headFile) {
                  $currentHead = Get-Content $headFile -Raw
                  if ($currentHead -ne $env:PROMPT_GIT_HEAD) {
                      $headChanged = $true
                  }
              }
          }

          if ($env:PROMPT_GIT_CACHE_DIR -eq $PWD.Path -and -not $headChanged) {
              $env:PROMPT_GIT_DIR_CACHED = $env:PROMPT_GIT_DIR
              $env:PROMPT_GIT_BRANCH_CACHED = $env:PROMPT_GIT_BRANCH
          } else {
              $env:PROMPT_GIT_DIR_CACHED = ""
              $env:PROMPT_GIT_BRANCH_CACHED = ""
          }

          $arguments = @(
              "prompt",
              "--terminal-width=$($Host.UI.RawUI.WindowSize.Width)",
              "--current-directory=$($PWD.Path)",
              "--current-directory-is-filesystem=$($PWD.Provider.Name -eq 'FileSystem')",
              "--last-command-state=$origDollarQuestion",
              "--last-command-exit-code=$LASTEXITCODE",
              "--last-command-duration=$( ([int](Get-History -Count 1).Duration.TotalMilliseconds).ToString([System.Globalization.CultureInfo]::InvariantCulture) )"
          )
  
          # Invoke <Prompt>
          $promptText = Invoke-Native -Executable '{{processName}}' -Arguments $arguments

          # Cache git info for next prompt
          $env:PROMPT_GIT_CACHE_DIR = $PWD.Path
          $env:PROMPT_GIT_DIR = $env:PROMPT_GIT_DIR_OUT
          $env:PROMPT_GIT_BRANCH = $env:PROMPT_GIT_BRANCH_OUT
          if ($env:PROMPT_GIT_DIR_OUT) {
              $headFile = Join-Path $env:PROMPT_GIT_DIR_OUT "HEAD"
              if (Test-Path $headFile) {
                  $env:PROMPT_GIT_HEAD = Get-Content $headFile -Raw
              }
          }

          # notify PSReadLine of a multiline prompt
          Set-PSReadLineOption -ExtraPromptLineCount (($promptText | Measure-Object -Line).Lines - 1)

          # Return the prompt
          $promptText
  
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
  }

  """;
}
