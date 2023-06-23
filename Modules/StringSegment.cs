using Cysharp.Text;

namespace Prompt.Modules;

internal sealed class StringSegment : Segment
{
    private readonly string _value;

    public StringSegment(string value)
    {
        _value = value;
    }

    public override int UnformattedLength => _value.Length;

    public override void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append(_value);
    }
}
