using System.Reflection;
using System.Text;
using Prompt.Segments;
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
            case ["--version"]:
            {
                string version = typeof(Program).Assembly
                                                .GetCustomAttribute<AssemblyInformationalVersionAttribute>()
                                                ?.InformationalVersion ?? "unknown";
                AnsiConsole.WriteLine(version);
                return;
            }

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

                ISegment[] line1;

                if (state.SimpleMode)
                {
                    long tsHost = SegmentTimer.Start();
                    var hostSegment = new HostSegment();
                    SegmentTimer.Record(nameof(HostSegment), tsHost);

                    long tsGit = SegmentTimer.Start();
                    var gitSegment = new GitSegment(state.CurrentDirectory.ToString());
                    SegmentTimer.Record(nameof(GitSegment), tsGit);

                    int maxPathLength = state.TerminalWidth
                                       - hostSegment.UnformattedLength
                                       - gitSegment.UnformattedLength
                                       - 1;

                    long tsPath = SegmentTimer.Start();
                    var pathSegment = new PathSegment(state.CurrentDirectory, state.CurrentDirectoryIsFileSystem, maxPathLength, simpleMode: true);
                    SegmentTimer.Record(nameof(PathSegment), tsPath);

                    line1 =
                    [
                        pathSegment,
                        gitSegment,
                        hostSegment,
                    ];
                }
                else
                {
                    var newLineSegment = new NewLineSegment();

                    long tsHost = SegmentTimer.Start();
                    var hostSegment = new HostSegment();
                    SegmentTimer.Record(nameof(HostSegment), tsHost);

                    long tsGit = SegmentTimer.Start();
                    var gitSegment = new GitSegment(state.CurrentDirectory.ToString());
                    SegmentTimer.Record(nameof(GitSegment), tsGit);

                    long tsExitCode = SegmentTimer.Start();
                    var lastCommandExitCodeSegment = new LastCommandExitCodeSegment(state.LastCommandExitCode, state.LastCommandState);
                    SegmentTimer.Record(nameof(LastCommandExitCodeSegment), tsExitCode);

                    long tsDuration = SegmentTimer.Start();
                    var lastCommandDurationSegment = new LastCommandDurationSegment(state.LastCommandDurationMs, Settings.LastCommandDurationThresholdMs);
                    SegmentTimer.Record(nameof(LastCommandDurationSegment), tsDuration);

                    long tsDateTime = SegmentTimer.Start();
                    var dateTimeSegment = new DateTimeSegment();
                    SegmentTimer.Record(nameof(DateTimeSegment), tsDateTime);

                    long tsOs = SegmentTimer.Start();
                    var osSegment = new OsSegment();
                    SegmentTimer.Record(nameof(OsSegment), tsOs);

                    var shellSegment = new StringSegment(" pwsh");

                    long tsPrompt = SegmentTimer.Start();
                    var promptSegment = new PromptSegment(Settings.Prompt);
                    SegmentTimer.Record(nameof(PromptSegment), tsPrompt);

                    int maxPathLength = state.TerminalWidth
                                       - hostSegment.UnformattedLength
                                       - gitSegment.UnformattedLength
                                       - lastCommandExitCodeSegment.UnformattedLength
                                       - lastCommandDurationSegment.UnformattedLength
                                       - dateTimeSegment.UnformattedLength
                                       - 3;

                    long tsPath = SegmentTimer.Start();
                    var pathSegment = new PathSegment(state.CurrentDirectory, state.CurrentDirectoryIsFileSystem, maxPathLength, simpleMode: false);
                    SegmentTimer.Record(nameof(PathSegment), tsPath);

                    var fillerWidth = state.TerminalWidth
                                      - hostSegment.UnformattedLength
                                      - pathSegment.UnformattedLength
                                      - gitSegment.UnformattedLength
                                      - lastCommandExitCodeSegment.UnformattedLength
                                      - lastCommandDurationSegment.UnformattedLength
                                      - dateTimeSegment.UnformattedLength
                                      - 2;

                    var fillerSegment = new StringSegment(fillerWidth <= 0 ? "" : new string(' ', fillerWidth));

                    line1 =
                    [
                        newLineSegment,

                        pathSegment,
                        gitSegment,
                        hostSegment,

                        fillerSegment,

                        lastCommandExitCodeSegment,
                        lastCommandDurationSegment,
                        dateTimeSegment,

                        newLineSegment,

                        osSegment,
                        shellSegment,
                        promptSegment,
                    ];
                }

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
                AnsiConsole.WriteLine("       Prompt --version");
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
                var typeName = segment.GetType().Name;
                AnsiConsole.MarkupInterpolated($"[yellow]{typeName} ({segment.UnformattedLength})[/]");

                if (SegmentTimer.Timings is { } timings && timings.TryGetValue(typeName, out var ms))
                {
                    AnsiConsole.MarkupInterpolated($" [grey][[{ms:F2}ms]][/]");
                }

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
