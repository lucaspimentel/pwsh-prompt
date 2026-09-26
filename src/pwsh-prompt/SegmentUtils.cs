using System.Text;
using Prompt.Segments;

namespace Prompt;

internal static class SegmentUtils
{
    // Number of Unicode code points, so astral-plane characters (surrogate
    // pairs) count as one column like every other character.
    public static int LengthInCodePoints(ReadOnlySpan<char> value)
    {
        var count = 0;

        for (var i = 0; i < value.Length;)
        {
            i += char.IsSurrogate(value[i]) && i + 1 < value.Length && char.IsSurrogatePair(value[i], value[i + 1]) ? 2 : 1;
            count++;
        }

        return count;
    }

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
