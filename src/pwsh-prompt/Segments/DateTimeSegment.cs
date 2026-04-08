using System.Text;

namespace Prompt.Segments;

internal readonly struct DateTimeSegment : ISegment
{
    private const string Format = " yyyy-MM-dd h:mm tt ";
    private readonly DateTimeOffset _now = DateTimeOffset.Now;

    public DateTimeSegment()
    {
    }

    public int UnformattedLength => _now.Hour % 12 < 10 ? Format.Length : Format.Length + 1;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.AppendSpanFormattable(_now, Format);
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
