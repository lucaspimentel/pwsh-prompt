using System.Text;

namespace Prompt.Segments;

internal readonly struct DateTimeSegment : ISegment
{
    private const string Format = " yyyy-MM-dd h:mm tt ";
    private readonly DateTimeOffset _now = DateTimeOffset.Now;

    public DateTimeSegment()
    {
    }

    // Hours 10, 11 and 12 render two digits (h: without a leading zero, and
    // 0 renders as 12 on the 12-hour clock), so the rendered length is one
    // more than the template.
    public int UnformattedLength => _now.Hour % 12 is 0 or >= 10 ? Format.Length + 1 : Format.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.AppendSpanFormattable(_now, Format);
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
