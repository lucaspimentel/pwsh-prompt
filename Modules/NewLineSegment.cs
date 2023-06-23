using Cysharp.Text;

namespace Prompt.Modules;

internal sealed class NewLineSegment : Segment
{
    private const string Value = "\r\n";

    public override int UnformattedLength => 0;

    public override void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append(Value);
    }
}
