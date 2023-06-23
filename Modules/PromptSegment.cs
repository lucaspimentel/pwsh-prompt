using Cysharp.Text;

namespace Prompt.Modules;

internal sealed class PromptSegment : Segment
{
    private readonly string _prompt;
    private readonly bool _lastCommandState;

    public override int UnformattedLength => _prompt.Length;

    public PromptSegment(string prompt, bool lastCommandState)
    {
        _prompt = prompt;
        _lastCommandState = lastCommandState;
    }

    public override void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append(_lastCommandState ? "[green]" : "[red]");
        sb.Append(_prompt);
        sb.Append("[/]");
    }
}
