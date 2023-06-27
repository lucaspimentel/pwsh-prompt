using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string Prefix = "  ";

    private readonly string _currentDirectory;
    private readonly int _userProfileDirectoryLength;

    public PathSegment(string currentDirectory)
    {
        string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        bool inUserHome = currentDirectory.StartsWith(userProfileDirectory, StringComparison.OrdinalIgnoreCase);

        _currentDirectory = currentDirectory;
        _userProfileDirectoryLength = inUserHome ? userProfileDirectory.Length : 0;
    }

    public int UnformattedLength => _userProfileDirectoryLength > 0 ?
                                        Prefix.Length + _currentDirectory.Length - _userProfileDirectoryLength + 1 :
                                        Prefix.Length + _currentDirectory.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        if (Path.Exists(_currentDirectory))
        {
            sb.Append("[blue]");
            sb.Append(Prefix);
        }
        else if (_currentDirectory.Equals("Env:\\", StringComparison.OrdinalIgnoreCase))
        {
            sb.Append("[yellow]");
            sb.Append(Prefix);
            sb.Append("[/][blue]");
        }
        else
        {
            sb.Append("[red]");
            sb.Append(Prefix);
            sb.Append("[/][blue]");
        }

        if (_userProfileDirectoryLength > 0)
        {
            sb.Append('~');
            sb.Append(_currentDirectory.AsSpan(_userProfileDirectoryLength));
        }
        else
        {
            sb.Append(_currentDirectory);
        }

        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
