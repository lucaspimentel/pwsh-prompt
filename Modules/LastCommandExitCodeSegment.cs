using System;
using System.Text;

namespace Prompt.Modules;

internal readonly struct LastCommandExitCodeSegment : ISegment
{
    private const string Prefix = "  "; // \ue654

    private readonly int _lastCommandExitCode;
    private readonly string _unformattedString;

    public int UnformattedLength => string.IsNullOrEmpty(_unformattedString) ? 0 : _unformattedString.Length;

    public LastCommandExitCodeSegment(int lastCommandExitCode, bool lastCommandState)
    {
        _lastCommandExitCode = lastCommandExitCode;

        if (_lastCommandExitCode is 0 || lastCommandState)
        {
            _unformattedString = "";
            return;
        }

        Span<char> buffer = stackalloc char[32];
        var builder = new ValueStringBuilder(buffer);

        try
        {
            AppendWithoutColors(ref builder);
            _unformattedString = builder.ToString();
        }
        finally
        {
            builder.Dispose();
        }
    }

    public void Append(ref ValueStringBuilder sb)
    {
        if (_lastCommandExitCode is 0)
        {
            return;
        }

        sb.Append("[red]");
        sb.Append(_unformattedString);
        sb.Append("[/]");
    }

    private void AppendWithoutColors(ref ValueStringBuilder sb)
    {
        if (_lastCommandExitCode is 0)
        {
            return;
        }

        sb.Append(Prefix);
        sb.AppendSpanFormattable(_lastCommandExitCode);
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
