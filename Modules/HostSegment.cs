using System.Net;
using System.Text;

namespace Prompt.Modules;

internal readonly struct HostSegment : ISegment
{
    private const string Prefix = "   ";

    private readonly string _hostname = Dns.GetHostName();

    public HostSegment()
    {
    }

    public int UnformattedLength  => _hostname.Length + Prefix.Length;

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
