using System;
using System.Text;

namespace Prompt.Modules;

internal readonly struct DateTimeSegment : ISegment
{
    private readonly DateTimeOffset _now = DateTimeOffset.Now;

    public DateTimeSegment()
    {
    }

    public int UnformattedLength => _now.Hour < 10 ? 22 : 23;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(" at ");
        sb.AppendSpanFormattable(_now, "yyyy-mm-dd hh:mm tt");
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
