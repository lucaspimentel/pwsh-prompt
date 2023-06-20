using System;
using System.IO;
using System.Text;
using Prompt;

switch (args)
{
    case ["init"]:
    {
        // Invoke-Expression (& 'C:\Program Files\starship\bin\starship.exe' init powershell --print-full-init | Out-String)
        // (@(& 'C:/Users/lucas/AppData/Local/Programs/oh-my-posh/bin/oh-my-posh.exe' init pwsh --config='' --print) -join "`n") | Invoke-Expression

        // $host.ui.RawUI.WindowTitle = 'pwsh'
        // [System.Console]::OutputEncoding = [System.Text.Encoding]::UTF8;
        // ❯    󰅒

        var processName = Environment.ProcessPath;

        Console.WriteLine(
            $@"
Write-Host 'Setting prompt...';

# Create a new dynamic module so we don't pollute the global namespace with our functions and variables
$null = New-Module lucas-prompt {{

    function Invoke-Native {{
        param($Executable, $Arguments)

        $startInfo = New-Object System.Diagnostics.ProcessStartInfo -ArgumentList $Executable -Property @{{
            StandardOutputEncoding = [System.Text.Encoding]::UTF8;
            RedirectStandardOutput = $true;
            RedirectStandardError = $true;
            CreateNoWindow = $true;
            UseShellExecute = $false;
            WorkingDirectory = $PWD.ProviderPath;
        }};

        # requires PowerShell 6+ (or 6.1+)
        foreach ($arg in $Arguments) {{
            $startInfo.ArgumentList.Add($arg);
        }}

        $process = [System.Diagnostics.Process]::Start($startInfo)

        # Read the output and error streams asynchronously
        # Avoids potential deadlocks when the child process fills one of the buffers
        # https://docs.microsoft.com/en-us/dotnet/api/system.diagnostics.process.standardoutput?view=net-6.0#remarks
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        [System.Threading.Tasks.Task]::WaitAll(@($stdout, $stderr))

        # stderr isn't displayed with this style of invocation
        # Manually write it to console
        if ($stderr.Result.Trim() -ne '') {{
            # Write-Error doesn't work here
            $host.ui.WriteErrorLine($stderr.Result)
        }}

        $stdout.Result;
    }}

    function Get-Hyperlink {{
        param(
            [Parameter(Mandatory, ValueFromPipeline = $True)]
            [string]$uri,
            [Parameter(ValueFromPipeline = $True)]
            [string]$name
        )

        if ("""" -eq $name) {{
            # if name not set, uri is used as the name of the hyperlink
            $name = $uri
        }}

        # if ($null -ne $env:WSL_DISTRO_NAME) {{
        #     # wsl conversion if needed
        #     $uri = &wslpath -m $uri
        # }}

        # return an ANSI formatted hyperlink
        $esc = [char]27
        return ""$esc]8;;$uri$esc\$name$esc]8;;$esc\""
    }}

    function global:Prompt {{
        $origDollarQuestion = $global:?
        $origLastExitCode = $global:LASTEXITCODE

        $arguments = @(
            ""prompt""
            ""--terminal-width=$($Host.UI.RawUI.WindowSize.Width)"",
            ""--last-command-state=$origDollarQuestion"",
            ""--last-command-duration=$((Get-History -Count 1).Duration.TotalMilliseconds)""
        )

        # Invoke <Prompt>
        $promptText = Invoke-Native -Executable '{processName}' -Arguments $arguments

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
        if ($global:? -ne $origDollarQuestion) {{
            if ($origDollarQuestion) {{
                # Simple command which will execute successfully and set $? = True without any other side affects.
                1+1
            }} else {{
                # Write-Error will set $? to False.
                # ErrorAction Ignore will prevent the error from being added to the $Error collection.
                Write-Error '' -ErrorAction 'Ignore'
            }}
        }}
    }}
}}
");

        break;
    }

    case ["prompt", ..]:
    {
        const string terminalWidthOption = "--terminal-width=";
        const string lastCommandStateOption = "--last-command-state=";
        const string lastCommandDurationOption = "--last-command-duration=";

        bool debug = Environment.GetEnvironmentVariable("DEBUG_PROMPT") == "1";
        int terminalWidth = 0;
        bool lastCommandState = true;
        TimeSpan lastCommandDuration = TimeSpan.Zero;

        foreach (string arg in args.AsSpan(1))
        {
            if (arg.StartsWith(terminalWidthOption, StringComparison.Ordinal))
            {
                if (int.TryParse(arg.AsSpan(terminalWidthOption.Length), out var result))
                {
                    terminalWidth = result;
                }
            }
            else if (arg.StartsWith(lastCommandStateOption, StringComparison.Ordinal))
            {
                if (bool.TryParse(arg.AsSpan(lastCommandStateOption.Length), out var result))
                {
                    lastCommandState = result;
                }
            }
            else if (arg.StartsWith(lastCommandDurationOption, StringComparison.Ordinal))
            {
                if (double.TryParse(arg.AsSpan(lastCommandDurationOption.Length), out var result))
                {
                    lastCommandDuration = TimeSpan.FromMilliseconds(result);
                }
            }
        }

        var fullPath = Directory.GetCurrentDirectory();
        var userProfile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        ReadOnlySpan<char> path;

        if (fullPath.StartsWith(userProfile, StringComparison.OrdinalIgnoreCase))
        {
            Span<char> buffer = stackalloc char[fullPath.Length - userProfile.Length + 1];
            buffer[0] = '~';
            fullPath.AsSpan(userProfile.Length).CopyTo(buffer[1..]);

            path = buffer.ToString();
        }
        else
        {
            path = fullPath;
        }

        Console.OutputEncoding = Encoding.UTF8;

        if (debug)
        {
            Console.WriteLine(Color.Yellow);
            Console.WriteLine($"Arguments = {string.Join(" ", args)}");
            Console.WriteLine($"terminalWidth = {terminalWidth}");
            Console.WriteLine($"lastCommandState = {lastCommandState}");
            Console.WriteLine($"lastCommandDuration = {lastCommandDuration}");
            Console.WriteLine(Color.Reset);
        }

        Console.WriteLine(Color.Reset);

        // line 1: path, git ... last command duration, current time
        var pathString = $" {path}";
        var pathSegment = $"{Color.Blue}{pathString}{Color.Reset}";

        var gitPath = Path.Combine(fullPath, ".git");
        var gitBranchName = GitInfo.GetBranchNameFrom(gitPath);
        string gitString;
        string gitSegment;

        if (gitBranchName.Length > 0)
        {
            gitString = $" in  {gitBranchName}";
            gitSegment = $"{Color.Magenta}{gitString}{Color.Reset}";
        }
        else
        {
            gitString = "";
            gitSegment = "";
        }

        string durationString = GetDurationString(
            lastCommandDuration,
            durationThreshold: 20,
            durationPrefix: "󰅒 ",
            force: debug);

        var durationSegment = !string.IsNullOrEmpty(durationString) ?
                                  $"{Color.Yellow}{durationString}{Color.Reset}" :
                                  "";

        var timeSegment = DateTime.Now.ToString(" 'at' yyyy-MM-dd h:mm tt");

        var fillerLength = terminalWidth - pathString.Length - gitString.Length - durationString.Length - timeSegment.Length;
        var fillerSegment = string.Create(fillerLength, fillerLength, (span, _) => span.Fill(' '));

        Console.WriteLine($"{pathSegment}{gitSegment}{fillerSegment}{durationSegment}{timeSegment}");

        // line 2: os, shell, prompt
        string lastCommandStateColor = lastCommandState ? Color.Green : Color.Red;
        Console.Write($" pwsh {lastCommandStateColor}❯{Color.Reset} ");

        break;
    }
}

static string GetDurationString(TimeSpan lastCommandDuration, double durationThreshold, string durationPrefix, bool force)
{
    if (lastCommandDuration.TotalMinutes >= 1)
    {
        return $"{durationPrefix}{lastCommandDuration:m'm 's's'}";
    }

    if (lastCommandDuration.TotalSeconds >= 1)
    {
        return $"{durationPrefix}{lastCommandDuration.TotalSeconds:0.0}s";
    }

    if (lastCommandDuration.TotalMilliseconds >= durationThreshold || force)
    {
        return $"{durationPrefix}{lastCommandDuration.Milliseconds}ms";
    }

    return "";
}

internal static class Color
{
    public const string Reset = "\x1b[0m";

    public const string Red = "\x1b[1;31m";
    public const string Green = "\x1b[1;32m";
    public const string Yellow = "\x1b[1;33m";
    public const string Blue = "\x1b[1;34m";
    public const string Magenta = "\x1b[1;35m";
}
