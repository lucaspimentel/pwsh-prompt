using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string Prefix = "   ";

    private readonly Microsoft.Extensions.Primitives.StringSegment _currentDirectoryDisplay;
    private readonly Microsoft.Extensions.Primitives.StringSegment _currentDirectoryExpanded;
    private readonly bool _isFileSystem;
    private readonly bool _isInUserHome;

    public PathSegment(Microsoft.Extensions.Primitives.StringSegment currentDirectory, bool isFileSystem)
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
                        _currentDirectoryDisplay = string.Concat(value, currentDirectorySpan[key.Length..]);
                        return;
                    }
                }
            }
        }

        string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        if (currentDirectory.StartsWith(userProfileDirectory, StringComparison.OrdinalIgnoreCase))
        {
            // remove user home from path, prepend "~" later
            _isInUserHome = true;
            _currentDirectoryDisplay = currentDirectory.Substring(userProfileDirectory.Length);
        }
    }

    public int UnformattedLength => _isInUserHome ?
                                        Prefix.Length + _currentDirectoryDisplay.Length + 1 : // "~"
                                        Prefix.Length + _currentDirectoryDisplay.Length;

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

        sb.Append(Prefix);

        if (_isInUserHome)
        {
            sb.Append('~');
        }

        sb.Append(_currentDirectoryDisplay);
        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
