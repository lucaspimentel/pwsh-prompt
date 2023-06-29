using System.Text;

namespace Prompt.Modules;

internal readonly struct GitSegment : ISegment
{
    private const string Prefix = " in  ";

    private readonly Microsoft.Extensions.Primitives.StringSegment _branchName;

    public GitSegment(string path)
    {
        _branchName = GitInfo.GetBranchName(path);
    }

    public int UnformattedLength => _branchName.Length == 0 ? 0 : Prefix.Length + _branchName.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        if (_branchName.Length == 0)
        {
            return;
        }

        sb.Append("[#ff7fff]");
        sb.Append(Prefix);
        sb.Append(_branchName);
        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
