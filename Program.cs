using System;
using System.Text;
using Prompt.Modules;
using Spectre.Console;

// Invoke-Expression (& 'C:\Program Files\starship\bin\starship.exe' init powershell --print-full-init | Out-String)
// (@(& 'C:/Users/lucas/AppData/Local/Programs/oh-my-posh/bin/oh-my-posh.exe' init pwsh --config='' --print) -join "`n") | Invoke-Expression

// $host.ui.RawUI.WindowTitle = 'pwsh'
// [System.Console]::OutputEncoding = [System.Text.Encoding]::UTF8;
// ❯    󰅒 󰥕 

namespace Prompt;

internal static class Program
{
    public static void Main(string[] args)
    {
        switch (args)
        {
            case ["init"]:
            {
#pragma warning disable Spectre1000
                string initString = Init.GetPowerShell(Environment.ProcessPath!);
                Console.WriteLine(initString);
#pragma warning restore Spectre1000
                return;
            }

            case ["prompt", ..]:
            {
                var state = Arguments.Parse(args.AsSpan(1));

                Console.OutputEncoding = Encoding.UTF8;

                AnsiConsole.Console.Profile.Width = state.TerminalWidth * 2;
                AnsiConsole.Console.Profile.Encoding = Encoding.UTF8;
                AnsiConsole.Console.Profile.Capabilities.Ansi = true;
                AnsiConsole.Console.Profile.Capabilities.Links = true;
                AnsiConsole.Console.Profile.Capabilities.Unicode = true;
                AnsiConsole.Console.Profile.Capabilities.ColorSystem = ColorSystem.TrueColor;
                AnsiConsole.Console.Profile.Capabilities.Interactive = false;
                AnsiConsole.Console.Profile.Capabilities.Legacy = false;

                if (Settings.Debug)
                {
                    //AnsiConsole.Markup("[yellow]");
                    AnsiConsole.WriteLine(@$"Literal arguments: ""{string.Join(" ", args)}""");
                    AnsiConsole.WriteLine($"Parsed arguments: {state}");
                    AnsiConsole.WriteLine($"Current directory: {Environment.CurrentDirectory}");
                    //AnsiConsole.Markup("[/]");
                }

                // Console.WriteLine($"{pathSegment}{gitSegment}{fillerSegment}{durationSegment}{dateTimeSegment}");
                // Console.WriteLine($"{osSegment}{shellSegment}{promptSegment}");

                var newLineSegment = new NewLineSegment();
                var hostSegment = new HostSegment();
                var gitSegment = new GitSegment(state.CurrentDirectory.ToString());
                var lastCommandDurationSegment = new LastCommandDurationSegment(state.LastCommandDurationMs, Settings.LastCommandDurationThresholdMs);
                var dateTimeSegment = new DateTimeSegment();
                var osSegment = new OsSegment();
                var shellSegment = new StringSegment(" pwsh");
                var promptSegment = new PromptSegment(Settings.Prompt, state.LastCommandState);

                int maxPathLength = state.TerminalWidth
                                   - hostSegment.UnformattedLength
                                   - gitSegment.UnformattedLength
                                   - lastCommandDurationSegment.UnformattedLength
                                   - dateTimeSegment.UnformattedLength
                                   - 3;

                var pathSegment = new PathSegment(state.CurrentDirectory, state.CurrentDirectoryIsFileSystem, maxPathLength);

                var fillerWidth = state.TerminalWidth
                                  - hostSegment.UnformattedLength
                                  - pathSegment.UnformattedLength
                                  - gitSegment.UnformattedLength
                                  - lastCommandDurationSegment.UnformattedLength
                                  - dateTimeSegment.UnformattedLength
                                  - 2;

                var fillerSegment = new StringSegment(fillerWidth <= 0 ? "" : new string(' ', fillerWidth));

                var line1 = new ISegment[]
                            {
                                newLineSegment,

                                pathSegment,
                                gitSegment,
                                hostSegment,

                                fillerSegment,

                                lastCommandDurationSegment,
                                dateTimeSegment,

                                newLineSegment,

                                osSegment,
                                shellSegment,
                                promptSegment,
                            };

                Span<char> buffer = stackalloc char[1024];
                var promptBuilder = new ValueStringBuilder(buffer);
                string prompt;

                try
                {
                    CombineSegments(ref promptBuilder, line1, state.TerminalWidth);
                    prompt = promptBuilder.ToString();
                }
                finally
                {
                    promptBuilder.Dispose();
                }

                AnsiConsole.Markup(prompt);
                return;
            }

            case null or []:
            {
                AnsiConsole.WriteLine("Usage: Prompt init");
                AnsiConsole.WriteLine("       Prompt prompt [arguments]");
                return;
            }
        }
    }

    private static void CombineSegments(ref ValueStringBuilder builder, ISegment[] segments, int width)
    {
        if (Settings.Debug)
        {
            AnsiConsole.WriteLine();

            foreach (var segment in segments)
            {
                AnsiConsole.MarkupInterpolated($"[yellow]{segment.GetType().Name} ({segment.UnformattedLength})[/]");

                if (segment is not NewLineSegment)
                {
                    AnsiConsole.Write(@$": ""{segment}""");
                }

                AnsiConsole.WriteLine();
            }
        }

        int remainingWidth = width;
        bool lineFull = false;

        foreach (var segment in segments)
        {
            if (segment is NewLineSegment)
            {
                // reset for next line
                remainingWidth = width;
                lineFull = false;
            }

            if (segment.UnformattedLength > remainingWidth)
            {
                lineFull = true;
            }

            if (!lineFull)
            {
                segment.Append(ref builder);
                remainingWidth -= segment.UnformattedLength;
            }
        }
    }
}
