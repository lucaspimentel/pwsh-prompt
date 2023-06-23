using System.Text;

namespace Prompt.Modules;

internal readonly struct PromptSegment : ISegment
{
    private readonly string _prompt;
    private readonly bool _lastCommandState;

    public int UnformattedLength => _prompt.Length + 2;

    public PromptSegment(string prompt, bool lastCommandState)
    {
        _prompt = prompt;
        _lastCommandState = lastCommandState;
    }

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(_lastCommandState ? "[green] " : "[red] ");
        sb.Append(_prompt);
        sb.Append(" [/]");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
