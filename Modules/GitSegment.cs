using System.IO;
using Cysharp.Text;

namespace Prompt.Modules;

internal readonly struct GitSegment : ISegment
{
    private const string Prefix = "in  ";

    private readonly Microsoft.Extensions.Primitives.StringSegment _branchName;

    public GitSegment()
    {
        string currentDirectory = Directory.GetCurrentDirectory();
        _branchName = GitInfo.GetBranchName(currentDirectory);
    }

    public  int UnformattedLength => _branchName.Length + Prefix.Length;

    public  void Append(ref Utf16ValueStringBuilder sb)
    {
        if (_branchName.Length == 0)
        {
            return;
        }

        sb.Append("[magenta]");
        sb.Append(Prefix);
        sb.Append(_branchName);
        sb.Append("[/]");
    }
}
