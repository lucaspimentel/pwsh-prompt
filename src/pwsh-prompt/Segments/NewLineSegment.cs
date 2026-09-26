using System.Text;

namespace Prompt.Segments;

internal readonly struct NewLineSegment : ISegment
{
    // Platform-appropriate line break; the previous hardcoded "\r\n" was
    // wrong on Unix.
    private static readonly string Value = Environment.NewLine;

    public int UnformattedLength => 0;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(Value);
    }
}
