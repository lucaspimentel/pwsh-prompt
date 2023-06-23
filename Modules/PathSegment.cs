using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
    private const string Prefix = "  ";

    private readonly string _currentDirectory;
    private readonly string _userProfileDirectory;
    private readonly bool _inUserHome;

    public PathSegment()
    {
        _currentDirectory = Directory.GetCurrentDirectory();
        _userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
        _inUserHome = _currentDirectory.StartsWith(_userProfileDirectory, StringComparison.OrdinalIgnoreCase);
    }

    public int UnformattedLength => _inUserHome ?
                                        Prefix.Length + _currentDirectory.Length - _userProfileDirectory.Length + 1 :
                                        Prefix.Length + _currentDirectory.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append("[blue]");
        sb.Append(Prefix);

        if (_inUserHome)
        {
            sb.Append('~');
            sb.Append(_currentDirectory.AsSpan(_userProfileDirectory.Length));
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
