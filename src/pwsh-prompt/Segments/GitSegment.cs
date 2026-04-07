using System;
using System.IO;
using System.Text;

namespace Prompt.Modules;

internal readonly struct GitSegment : ISegment
{
    private const string Prefix = " in  ";
    private const string PrPrefix = " #";

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

    public int UnformattedLength => _branchName.Length == 0
        ? 0
        : Prefix.Length + _branchName.Length + (_prNumber != null ? PrPrefix.Length + _prNumber.Length : 0);

    public void Append(ref ValueStringBuilder sb)
    {
        if (_branchName.Length == 0)
        {
            return;
        }

        sb.Append("[#ff7fff]");
        sb.Append(Prefix);
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
