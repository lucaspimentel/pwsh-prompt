using Cysharp.Text;

namespace Prompt.Modules;

internal readonly struct StringSegment : ISegment
{
    private readonly string _value;

    public StringSegment(string value)
    {
        _value = value;
    }

    public int UnformattedLength => _value.Length;

    public void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append(_value);
    }
}
