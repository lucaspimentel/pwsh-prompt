using System;
using Cysharp.Text;

namespace Prompt.Modules;

internal sealed class FillerSegment : Segment
{
    public override int UnformattedLength { get; }

    public FillerSegment(int length)
    {
        UnformattedLength = length < 0 ? 0 : length;
    }

    public override void Append(ref Utf16ValueStringBuilder sb)
    {
        Span<char> span = stackalloc char[UnformattedLength];
        span.Fill(' ');

        sb.Append(span);
    }
}
