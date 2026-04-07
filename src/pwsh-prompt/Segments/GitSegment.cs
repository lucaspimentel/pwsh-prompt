using System.Text;

namespace Prompt.Segments;

internal readonly struct GitSegment : ISegment
{
    private const string BranchPrefix = "   ";
    private const string PrPrefix = " PR#";

    private readonly Microsoft.Extensions.Primitives.StringSegment _branchName;
    private readonly string? _prNumber;

    public GitSegment(string path)
    {
        if (Path.Exists(path))
        {
            _branchName = GitInfo.GetBranchName(path);

            var prNumber = Environment.GetEnvironmentVariable("PROMPT_PR_NUMBER_CACHED");
            if (!string.IsNullOrEmpty(prNumber))
            {
                _prNumber = prNumber;
            }
        }
    }

    public int UnformattedLength
    {
        get
        {
            if (_branchName.Length == 0)
            {
                return 0;
            }
            else
            {
                int prPrefixLength = _prNumber != null ? PrPrefix.Length + _prNumber.Length : 0;
                return BranchPrefix.Length + _branchName.Length + prPrefixLength;
            }
        }
    }

    public void Append(ref ValueStringBuilder sb)
    {
        if (_branchName.Length == 0)
        {
            return;
        }

        sb.Append("[#ff7fff]");
        sb.Append(BranchPrefix);
        sb.Append(_branchName);

        if (_prNumber != null)
        {
            sb.Append(PrPrefix);
            sb.Append(_prNumber);
        }

        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
