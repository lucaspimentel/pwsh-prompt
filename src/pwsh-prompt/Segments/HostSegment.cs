using System.Text;

namespace Prompt.Segments;

internal readonly struct HostSegment : ISegment
{
    private const string Prefix = "    ";

    private readonly string _hostname = Environment.MachineName.ToLowerInvariant();

    public HostSegment()
    {
    }

    // Code-point length of the hostname, plus the trailing space that Append
    // emits inside the colored range (previously not counted, which
    // under-reserved one column of layout width).
    public int UnformattedLength => SegmentUtils.LengthInCodePoints(_hostname.AsSpan()) + Prefix.Length + 1;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append("[blue]");
        sb.Append(Prefix);
        sb.Append(_hostname);
        sb.Append(" [/]");
    }

    public override string ToString()
    {
        return _hostname;
    }
}
