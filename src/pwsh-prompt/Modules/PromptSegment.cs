using System.Text;

namespace Prompt.Modules;

internal readonly struct PromptSegment : ISegment
{
    private readonly string _prompt;

    public int UnformattedLength => _prompt.Length;

    public PromptSegment(string prompt)
    {
        _prompt = prompt;
    }

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append("[lime]");
        sb.Append(_prompt);
        sb.Append("[/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
