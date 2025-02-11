using System;
using System.IO;
using System.Text;
using Spectre.Console;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string DefaultPrefix = "   ";
    private const string GitPrefix = "   ";

    private readonly Microsoft.Extensions.Primitives.StringSegment _currentDirectoryDisplay;
    private readonly Microsoft.Extensions.Primitives.StringSegment _currentDirectoryExpanded;
    private readonly bool _isFileSystem;
    private readonly bool _isInUserHome;
    private readonly bool _isTruncated;
    private readonly bool _isGitRepo;

    public PathSegment(Microsoft.Extensions.Primitives.StringSegment currentDirectory, bool isFileSystem, int maxPathLength)
    {
        _currentDirectoryDisplay = _currentDirectoryExpanded = currentDirectory;
        _isFileSystem = isFileSystem;

        if (!isFileSystem)
        {
            return;
        }

        var processDirectory = Path.GetDirectoryName(Environment.ProcessPath);

        if (processDirectory is null)
        {
            return;
        }

        /*
        string mapDefinitionFilename = Path.Combine(processDirectory, "prompt-path-mappings.txt");

        if (File.Exists(mapDefinitionFilename))
        {
            var lines = File.ReadLines(mapDefinitionFilename);

            foreach (string line in lines)
            {
                var lineSpan = line.AsSpan();
                var split = line.IndexOf('|');

                if (split > 0)
                {
                    var key = lineSpan[..split];
                    var currentDirectorySpan = currentDirectory.AsSpan().TrimEnd(@"/\");

                    // if current directory equals "key", or starts with "key/" or "key\", replace "key" with "value"
                    if (currentDirectorySpan.StartsWith(key, StringComparison.OrdinalIgnoreCase) &&
                        (currentDirectorySpan.Length == key.Length || currentDirectorySpan[key.Length] is '/' or '\\'))
                    {
                        var value = lineSpan[(split + 1)..].TrimEnd(@"/\");
                        _currentDirectoryDisplay = ShortenPath(string.Concat(value, currentDirectorySpan[key.Length..]), maxPathLength);
                        return;
                    }
                }
            }
        }
        */

        if (GitInfo.TryFindGitFolder(currentDirectory, out var gitDirectory))
        {
            // currentDirectory = /path/to/repo[/child]
            // gitDirectory = /path/to/repo/.git
            // ----------------------------------------------
            // repositoryDirectory = /path/to/repo[/child]
            // repositoryParentDirectory = /path/to

            if (Path.GetDirectoryName(gitDirectory) is { } repositoryDirectory &&
                Path.GetDirectoryName(repositoryDirectory) is { } repositoryParentDirectory)
            {
                string displayPath = Path.GetRelativePath(repositoryParentDirectory, currentDirectory.ToString());

                if (displayPath.Length > 0)
                {
                    _isGitRepo = true;
                    _currentDirectoryDisplay = displayPath;
                }
            }
        }
        else
        {
            string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            if (Settings.Debug)
            {
                AnsiConsole.MarkupLineInterpolated($"[yellow]userProfileDirectory: {userProfileDirectory}[/]");
            }

            if (currentDirectory.StartsWith(userProfileDirectory, StringComparison.OrdinalIgnoreCase))
            {
                // remove user home from path, prepend "~" later
                _isInUserHome = true;
                _currentDirectoryDisplay = currentDirectory.Subsegment(userProfileDirectory.Length);
            }
        }

        if (Settings.Debug)
        {
            AnsiConsole.MarkupLineInterpolated($"[yellow]displayPath before truncating: {_currentDirectoryDisplay}[/]");
        }

        _isTruncated = TryShortenPath(_currentDirectoryDisplay, maxPathLength, out _currentDirectoryDisplay);

        if (Settings.Debug)
        {
            AnsiConsole.MarkupLineInterpolated($"[yellow]displayPath after truncating: {_currentDirectoryDisplay}[/]");
        }
    }

    public int UnformattedLength
    {
        get
        {
            var length = _currentDirectoryDisplay.Length + (_isGitRepo ? GitPrefix.Length : DefaultPrefix.Length);

            return (_isInUserHome, _isTruncated) switch
            {
                (true, true) => length + 5,  // "~/..."
                (true, false) => length + 1, // "~"
                (false, true) => length + 3, // "..."
                _ => length
            };
        }
    }

    private static bool TryShortenPath(
        Microsoft.Extensions.Primitives.StringSegment path,
        int maxPathLength,
        out Microsoft.Extensions.Primitives.StringSegment truncatedSegment)
    {
        truncatedSegment = path;

        if (path.Length <= maxPathLength)
        {
            return false;
        }

        Span<char> separator = stackalloc char[2];
        separator[0] = Path.DirectorySeparatorChar;
        separator[1] = Path.AltDirectorySeparatorChar;

        for (var i = 0; i < path.Length; i++)
        {
            if (path[i] == separator[0] || path[i] == separator[1])
            {
                var newLength = path.Length - i + 3;

                if (0 < newLength && newLength <= maxPathLength)
                {
                    truncatedSegment = path.Subsegment(i);
                    return true;
                }
            }
        }

        // couldn't find a separator to truncate at
        return false;
    }

    public void Append(ref ValueStringBuilder sb)
    {
        if (_isFileSystem && !Path.Exists(_currentDirectoryExpanded.ToString()))
        {
            // is file system, but doesn't exist
            // e.g. directory was deleted from under us
            sb.Append("[red]");
        }
        else
        {
            sb.Append("[aqua]");
        }

        sb.Append(_isGitRepo ? GitPrefix : DefaultPrefix);

        if (_isInUserHome)
        {
            sb.Append('~');
        }

        if (_isInUserHome && _isTruncated)
        {
            sb.Append(Path.PathSeparator);
        }

        if (_isTruncated)
        {
            sb.Append("...");
        }

        sb.Append(_currentDirectoryDisplay);
        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
