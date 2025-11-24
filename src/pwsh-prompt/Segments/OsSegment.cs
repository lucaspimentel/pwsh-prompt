using System.Runtime.InteropServices;
using System.Text;

namespace Prompt.Modules;

internal readonly struct OsSegment : ISegment
{
    private readonly string _value;

    public OsSegment()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            _value = "  ";
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            _value = "  ";
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            _value = "  ";
        }
        else
        {
            _value = " OS? ";
        }
    }

    public int UnformattedLength => _value.Length;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(_value);
    }

    public override string ToString()
    {
        return _value;
    }
}
