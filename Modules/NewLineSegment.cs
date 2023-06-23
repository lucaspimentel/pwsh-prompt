using Cysharp.Text;

namespace Prompt.Modules;

internal readonly struct NewLineSegment : ISegment
{
    private const string Value = "\r\n";

    public int UnformattedLength => 0;

    public void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append(Value);
    }
}
