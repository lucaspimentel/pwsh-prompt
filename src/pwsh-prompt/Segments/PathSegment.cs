using System.Text;

namespace Prompt.Segments;

internal readonly struct PathSegment : ISegment
{
    private const string DefaultPrefix = "   ";
    private const string GitPrefix = "   ";

    private readonly Microsoft.Extensions.Primitives.StringSegment _currentDirectoryDisplay;
    private readonly bool _isInUserHome;
    private readonly bool _isTruncated;
    private readonly bool _isGitRepo;

    public PathSegment(Microsoft.Extensions.Primitives.StringSegment currentDirectory, bool isFileSystem, int maxPathLength, bool simpleMode)
    {
        _currentDirectoryDisplay = currentDirectory;

        if (!isFileSystem)
        {
            return;
        }


        if (GitInfo.TryFindGitFolder(currentDirectory, out var gitDirectory))
        {
            _isGitRepo = true;

            // In simple mode, keep the full path instead of shortening to repo root
            if (!simpleMode)
            {
                // currentDirectory = /path/to/repo[/child]
                // gitDirectory = /path/to/repo/.git or /path/to/repo/.git/worktrees/name
                // ----------------------------------------------
                // repositoryDirectory = /path/to/repo or /path/to/repo/.git/worktrees
                // repositoryParentDirectory = /path/to or /path/to/repo/.git

                var repositoryDirectory = Path.GetDirectoryName(gitDirectory);

                // For worktrees, gitDirectory is .git/worktrees/<name>, so we need to go up twice more
                if (repositoryDirectory != null && repositoryDirectory.EndsWith("worktrees", StringComparison.Ordinal))
                {
                    repositoryDirectory = Path.GetDirectoryName(Path.GetDirectoryName(repositoryDirectory));
                }

                if (repositoryDirectory != null)
                {
                    var repositoryParentDirectory = Path.GetDirectoryName(repositoryDirectory);
                    if (repositoryParentDirectory != null)
                    {
                        string displayPath = Path.GetRelativePath(repositoryParentDirectory, currentDirectory.ToString());

                        if (displayPath.Length > 0)
                        {
                            _currentDirectoryDisplay = displayPath;
                        }
                    }
                }
            }
        }
        else
        {
            string userProfileDirectory = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

            if (Settings.Debug)
            {
                DebugOut.YellowLine($"userProfileDirectory: {userProfileDirectory}");
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
            DebugOut.YellowLine($"displayPath before truncating: {_currentDirectoryDisplay}");
        }

        _isTruncated = TryShortenPath(_currentDirectoryDisplay, maxPathLength, out _currentDirectoryDisplay);

        if (Settings.Debug)
        {
            DebugOut.YellowLine($"displayPath after truncating: {_currentDirectoryDisplay}");
        }
    }

    public int UnformattedLength
    {
        get
        {
            var length = SegmentUtils.LengthInCodePoints(_currentDirectoryDisplay.AsSpan()) + (_isGitRepo ? GitPrefix.Length : DefaultPrefix.Length);

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

        // Compare and cut in code points so astral-plane characters count as
        // one column. Separators are always BMP, so the cut point is a valid
        // code-point boundary.
        var totalCodePoints = SegmentUtils.LengthInCodePoints(path.AsSpan());

        if (totalCodePoints <= maxPathLength)
        {
            return false;
        }

        var separator = Path.DirectorySeparatorChar;
        var altSeparator = Path.AltDirectorySeparatorChar;
        var codePointsSeen = 0;

        for (var i = 0; i < path.Length;)
        {
            if (path[i] == separator || path[i] == altSeparator)
            {
                var newLength = totalCodePoints - codePointsSeen + 3;

                if (0 < newLength && newLength <= maxPathLength)
                {
                    truncatedSegment = path.Subsegment(i);
                    return true;
                }
            }

            i += char.IsSurrogate(path[i]) && i + 1 < path.Length && char.IsSurrogatePair(path[i], path[i + 1]) ? 2 : 1;
            codePointsSeen++;
        }

        // couldn't find a separator to truncate at
        return false;
    }

    public void Append(ref ValueStringBuilder sb)
    {
        // Skip Path.Exists check for performance - rare edge case not worth the I/O cost
        sb.Append("[aqua]");

        sb.Append(_isGitRepo ? GitPrefix : DefaultPrefix);

        if (_isInUserHome)
        {
            sb.Append('~');
        }

        if (_isInUserHome && _isTruncated)
        {
            // Directory separator; this previously appended Path.PathSeparator
            // (';'), producing "~;..." on Windows.
            sb.Append(Path.DirectorySeparatorChar);
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
