using System;
using System.Text;
using Prompt.Modules;

namespace Prompt;

internal static class SegmentUtils
{
    public static string ToString<TSegment>(TSegment segment) where TSegment : ISegment
    {
        Span<char> buffer = stackalloc char[256];
        var builder = new ValueStringBuilder(buffer);

        try
        {
            segment.Append(ref builder);
            return builder.ToString();
        }
        finally
        {
            builder.Dispose();
        }
    }
}
