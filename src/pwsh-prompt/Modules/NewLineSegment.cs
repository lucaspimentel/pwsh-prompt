using System.Text;

namespace Prompt.Modules;

internal readonly struct NewLineSegment : ISegment
{
    private const string Value = "\r\n";

    public int UnformattedLength => 0;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(Value);
    }
}
