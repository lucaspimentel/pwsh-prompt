using System;
using Cysharp.Text;

namespace Prompt.Modules;

internal readonly struct DateTimeSegment : ISegment
{
    private readonly DateTimeOffset _now = DateTimeOffset.Now;

    public DateTimeSegment()
    {
    }

    public int UnformattedLength => _now.Hour < 10 ? 21 : 22;

    public void Append(ref Utf16ValueStringBuilder sb)
    {
        sb.Append("at ");
        sb.Append(_now, "yyyy-mm-dd hh:mm tt");
    }
}
