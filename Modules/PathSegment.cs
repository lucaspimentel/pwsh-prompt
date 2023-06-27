using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string Prefix = "  ";

    private readonly string _currentDirectory;
    private readonly string _displayDirectory;
    private readonly bool _isFileSystem;
    private readonly bool _isInUserHome;

    public PathSegment(
        Microsoft.Extensions.Primitives.StringSegment currentDirectory,
        Microsoft.Extensions.Primitives.StringSegment currentDirectoryProvider)
    {
        _currentDirectory = _displayDirectory = currentDirectory.ToString();
        bool isFileSystem = _isFileSystem = currentDirectoryProvider.Equals(@"Microsoft.PowerShell.Core\FileSystem", StringComparison.OrdinalIgnoreCase);

        if (isFileSystem)
        {
            string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            if (currentDirectory.StartsWith(userProfileDirectory, StringComparison.OrdinalIgnoreCase))
            {
                // remove user home from path, replace with "~" later
                _isInUserHome = true;
                _displayDirectory = currentDirectory.Substring(userProfileDirectory.Length);
            }
        }
    }

    public int UnformattedLength => _isInUserHome ?
                                        Prefix.Length + _displayDirectory.Length + 1 : // "~"
                                        Prefix.Length + _displayDirectory.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        if (!_isFileSystem)
        {
            // e.g. "Env:\", "Function:\", "Variable:\"
            sb.Append("[yellow]");
        }
        else if (!Path.Exists(_currentDirectory))
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

        sb.Append(_displayDirectory);
        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
