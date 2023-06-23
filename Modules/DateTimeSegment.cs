using System;
using Cysharp.Text;

namespace Prompt.Modules;

internal sealed class DateTimeSegment : Segment
{
    private readonly DateTimeOffset _now = DateTimeOffset.Now;

    public override int UnformattedLength => _now.Hour < 10 ? 21 : 22;

    public override void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append("at ");
        sb.Append(_now, "yyyy-mm-dd hh:mm tt");
    }
}
