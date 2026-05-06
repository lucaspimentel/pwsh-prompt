using System.Text;

namespace Prompt.Segments;

internal readonly struct GitSegment : ISegment
{
    private const string BranchPrefix = "  \ue725 "; // branch, ,    󰘬 
    private const string PrIconOpen   = "  \uea64 "; // open PR, 
    private const string PrIconClosed = "  \uebda "; // closed PR, 
    private const string PrIconDraft  = "  \uebdb "; // draft PR, 

    private readonly Microsoft.Extensions.Primitives.StringSegment _branchName;
    private readonly string? _prNumber;
    private readonly string _prIcon;

    public GitSegment(string path)
    {
        if (Path.Exists(path))
        {
            _branchName = GitInfo.GetBranchName(path);

            var prNumber = Environment.GetEnvironmentVariable("PROMPT_PR_NUMBER_CACHED");
            if (!string.IsNullOrEmpty(prNumber))
            {
                _prNumber = prNumber;
                var prState = Environment.GetEnvironmentVariable("PROMPT_PR_STATE_CACHED");
                _prIcon = prState switch
                {
                    "closed" => PrIconClosed,
                    "draft"  => PrIconDraft,
                    _        => PrIconOpen,
                };
            }
            else
            {
                _prIcon = PrIconOpen;
            }
        }
        else
        {
            _prIcon = PrIconOpen;
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
                int prPrefixLength = _prNumber != null ? _prIcon.Length + _prNumber.Length : 0;
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
            sb.Append(_prIcon);
            sb.Append(_prNumber);
        }

        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
