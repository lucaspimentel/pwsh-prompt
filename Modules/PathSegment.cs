using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string Prefix = "   ";

    private readonly string _currentDirectoryDisplay;
    private readonly string _currentDirectoryExpanded;
    private readonly bool _isFileSystem;
    private readonly bool _isInUserHome;

    public PathSegment(Microsoft.Extensions.Primitives.StringSegment currentDirectory, bool isFileSystem)
    {
        _currentDirectoryDisplay = _currentDirectoryExpanded = currentDirectory.ToString();
        _isFileSystem = isFileSystem;

        if (isFileSystem)
        {
            string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            if (currentDirectory.StartsWith(userProfileDirectory, StringComparison.OrdinalIgnoreCase))
            {
                // remove user home from path, prepend "~" later
                _isInUserHome = true;
                _currentDirectoryDisplay = currentDirectory.Substring(userProfileDirectory.Length);
            }
        }
    }

    public int UnformattedLength => _isInUserHome ?
                                        Prefix.Length + _currentDirectoryDisplay.Length + 1 : // "~"
                                        Prefix.Length + _currentDirectoryDisplay.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        if (_isFileSystem && !Path.Exists(_currentDirectoryExpanded))
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
