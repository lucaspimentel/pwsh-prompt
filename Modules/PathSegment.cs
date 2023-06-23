using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct PathSegment : ISegment
{
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
                                                 _currentDirectory.Length - _userProfileDirectory.Length + 3 :
                                                 _currentDirectory.Length + 2;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append("[blue] ");

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
}
