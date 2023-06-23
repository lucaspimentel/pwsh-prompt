using System;
using Cysharp.Text;

namespace Prompt.Modules;

internal readonly struct FillerSegment : ISegment
{
    public  int UnformattedLength { get; }

    public FillerSegment(int length)
    {
        UnformattedLength = length < 0 ? 0 : length;
    }

    public  void Append(ref Utf16ValueStringBuilder sb)
    {
        Span<char> span = stackalloc char[UnformattedLength];
        span.Fill(' ');

        sb.Append(span);
    }
}
