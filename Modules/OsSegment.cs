using System.Runtime.InteropServices;
using System.Text;

namespace Prompt.Modules;

internal readonly struct OsSegment : ISegment
{
    private readonly string _value = GetOsString();

    public OsSegment()
    {
    }

    public int UnformattedLength => GetOsString().Length;

    public void Append(ref ValueStringBuilder sb)
    {
        sb.Append(_value);
    }

    private static string GetOsString()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            return "";
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            return "";
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
        {
            return "";
        }

        return "Unknown OS";
    }

    public override string ToString()
    {
        return _value;
    }
}
