using System;
using System.Globalization;
using Microsoft.Extensions.Primitives;

namespace Prompt;

public readonly record struct Arguments(int TerminalWidth, StringSegment CurrentDirectory, bool CurrentDirectoryIsFileSystem, int LastCommandDurationMs, bool LastCommandState)
{
    private const string TerminalWidthOption = "--terminal-width=";
    private const string CurrentDirectoryOption = "--current-directory=";
    private const string CurrentDirectoryIsFileSystemOption = "--current-directory-is-filesystem=";
    private const string LastCommandDurationOption = "--last-command-duration=";
    private const string LastCommandStateOption = "--last-command-state=";

    public static Arguments Parse(Span<string> args)
    {
        int terminalWidth = 0;
        StringSegment currentDirectory = "";
        bool currentDirectoryIsFileSystem = true;
        int lastCommandDurationMs = 0;
        bool lastCommandState = true;

        foreach (string arg in args)
        {
            if (arg.StartsWith(TerminalWidthOption, StringComparison.Ordinal))
            {
                if (int.TryParse(arg.AsSpan(TerminalWidthOption.Length), out var result))
                {
                    terminalWidth = result;
                }
            }
            else if (arg.StartsWith(CurrentDirectoryOption, StringComparison.Ordinal))
            {
                currentDirectory = new StringSegment(arg, CurrentDirectoryOption.Length, arg.Length - CurrentDirectoryOption.Length);
            }
            else if (arg.StartsWith(CurrentDirectoryIsFileSystemOption, StringComparison.Ordinal))
            {
                if (bool.TryParse(arg.AsSpan(CurrentDirectoryIsFileSystemOption.Length), out var result))
                {
                    currentDirectoryIsFileSystem = result;
                }
            }
            else if (arg.StartsWith(LastCommandDurationOption, StringComparison.Ordinal))
            {
                if (int.TryParse(arg.AsSpan(LastCommandDurationOption.Length), CultureInfo.InvariantCulture, out var result))
                {
                    lastCommandDurationMs = result;
                }
            }
            else if (arg.StartsWith(LastCommandStateOption, StringComparison.Ordinal))
            {
                if (bool.TryParse(arg.AsSpan(LastCommandStateOption.Length), out var result))
                {
                    lastCommandState = result;
                }
            }
        }

        if (terminalWidth == 0)
        {
            try
            {
                terminalWidth = Console.WindowWidth;
            }
            catch
            {
                terminalWidth = 80;
            }
        }

        if (currentDirectory.Length == 0)
        {
            currentDirectory = Environment.CurrentDirectory;
        }

        return new Arguments(terminalWidth, currentDirectory, currentDirectoryIsFileSystem, lastCommandDurationMs, lastCommandState);
    }
}
