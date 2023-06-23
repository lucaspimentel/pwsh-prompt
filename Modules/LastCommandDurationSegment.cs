using System;
using System.Text;

namespace Prompt.Modules;

internal readonly struct LastCommandDurationSegment : ISegment
{
    private readonly int _lastCommandDurationMs;
    private readonly int _thresholdMs;
    private readonly string _unformattedString;

    public int UnformattedLength => string.IsNullOrEmpty(_unformattedString) ? 0 : _unformattedString.Length - 1;

    public LastCommandDurationSegment(int lastCommandDurationMs, int thresholdMs)
    {
        _lastCommandDurationMs = lastCommandDurationMs;
        _thresholdMs = thresholdMs;

        if (_lastCommandDurationMs < _thresholdMs)
        {
            _unformattedString = "";
            return;
        }

        Span<char> buffer = stackalloc char[128];
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
        if (_lastCommandDurationMs < _thresholdMs)
        {
            return;
        }

        sb.Append("[yellow]");
        sb.Append(_unformattedString);
        sb.Append("[/]");
    }

    private void AppendWithoutColors(ref ValueStringBuilder sb)
    {
        if (_lastCommandDurationMs < _thresholdMs)
        {
            return;
        }

        sb.Append("󰥕 ");

        switch (_lastCommandDurationMs)
        {
            case >= 60_000:
                // 1,440m 59s
                int minutes = _lastCommandDurationMs / 60_000;
                int seconds = (_lastCommandDurationMs % 60_000) / 1_000;

                if (minutes >= 1000)
                {
                    sb.AppendSpanFormattable(minutes, "N");
                }
                else
                {
                    sb.AppendSpanFormattable(minutes);
                }

                sb.Append("m ");
                sb.AppendSpanFormattable(seconds);
                sb.Append('s');
                break;

            case >= 1_000:
                // 59.9s
                double totalSeconds = (_lastCommandDurationMs % 60_000) / 1_000.0;
                sb.AppendSpanFormattable(totalSeconds, "0.#");
                sb.Append('s');
                break;

            default:
                // 999ms
                sb.AppendSpanFormattable(_lastCommandDurationMs);
                sb.Append("ms");
                break;
        }
    }

    public override string ToString()
    {
        return SegmentUtils.ToString(this);
    }
}
