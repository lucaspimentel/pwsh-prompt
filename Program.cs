using System;
using System.Text;
using Prompt.Modules;
using Spectre.Console;

// Invoke-Expression (& 'C:\Program Files\starship\bin\starship.exe' init powershell --print-full-init | Out-String)
// (@(& 'C:/Users/lucas/AppData/Local/Programs/oh-my-posh/bin/oh-my-posh.exe' init pwsh --config='' --print) -join "`n") | Invoke-Expression

// $host.ui.RawUI.WindowTitle = 'pwsh'
// [System.Console]::OutputEncoding = [System.Text.Encoding]::UTF8;
// ❯    󰅒 󰥕

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

                AnsiConsole.Console.Profile.Width = state.TerminalWidth + 10;
                AnsiConsole.Console.Profile.Encoding = Encoding.UTF8;
                AnsiConsole.Console.Profile.Capabilities.Ansi = true;
                AnsiConsole.Console.Profile.Capabilities.Links = true;
                AnsiConsole.Console.Profile.Capabilities.Unicode = true;
                AnsiConsole.Console.Profile.Capabilities.ColorSystem = ColorSystem.TrueColor;
                AnsiConsole.Console.Profile.Capabilities.Interactive = false;
                AnsiConsole.Console.Profile.Capabilities.Legacy = false;

                if (Settings.Debug)
                {
                    AnsiConsole.Foreground = Color.Yellow;
                    AnsiConsole.WriteLine(@$"Literal arguments: ""{string.Join(" ", args)}""");
                    AnsiConsole.WriteLine($"Parsed arguments: {state}");
                    AnsiConsole.ResetColors();
                }

                // Console.WriteLine($"{pathSegment}{gitSegment}{fillerSegment}{durationSegment}{dateTimeSegment}");
                // Console.WriteLine($"{osSegment}{shellSegment}{promptSegment}");

                var spaceSegment = new StringSegment(" ");
                var newLineSegment = new NewLineSegment();
                var pathSegment = new PathSegment();
                var gitSegment = new GitSegment();
                var lastCommandDurationSegment = new LastCommandDurationSegment(state.LastCommandDurationMs, Settings.LastCommandDurationThresholdMs);
                var dateTimeSegment = new DateTimeSegment();
                var osSegment = new OsSegment();
                var shellSegment = new StringSegment("pwsh");
                var promptSegment = new PromptSegment(Settings.Prompt, state.LastCommandState);

                var fillerWidth = state.TerminalWidth - pathSegment.UnformattedLength - gitSegment.UnformattedLength - lastCommandDurationSegment.UnformattedLength - dateTimeSegment.UnformattedLength - 5;
                var fillerSegment = new StringSegment(new string(' ', fillerWidth));

                var line1 = new ISegment[]
                            {
                                newLineSegment,

                                spaceSegment,
                                pathSegment,

                                spaceSegment,
                                gitSegment,

                                fillerSegment,

                                spaceSegment,
                                lastCommandDurationSegment,

                                spaceSegment,
                                dateTimeSegment,

                                newLineSegment,

                                spaceSegment,
                                osSegment,

                                spaceSegment,
                                shellSegment,

                                spaceSegment,
                                promptSegment,
                                spaceSegment,
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

                if (Settings.Debug)
                {
                    AnsiConsole.WriteLine(prompt);
                    AnsiConsole.WriteLine();
                }

                AnsiConsole.Markup(prompt);
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
                if (segment is NewLineSegment)
                {
                    AnsiConsole.MarkupInterpolated(@$"[yellow]{segment.GetType().Name} ({segment.UnformattedLength})[/]");
                }
                else
                {
                    AnsiConsole.MarkupInterpolated(@$"[yellow]{segment.GetType().Name} ({segment.UnformattedLength})[/]: ""{segment.ToString()}""");
                }

                AnsiConsole.WriteLine();
            }
        }

        int remainingWidth = width;

        foreach (var segment in segments)
        {
            if (segment is NewLineSegment)
            {
                remainingWidth = width;
            }

            if (segment.UnformattedLength > remainingWidth)
            {
                break;
            }

            segment.Append(ref builder);
            remainingWidth -= segment.UnformattedLength;
        }
    }
}
