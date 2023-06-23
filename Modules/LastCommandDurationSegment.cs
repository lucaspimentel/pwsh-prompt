using Cysharp.Text;

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

        var builder = ZString.CreateStringBuilder(notNested: true);

        try
        {
            AppendNoColors(ref builder);
            _unformattedString = builder.ToString();
        }
        finally
        {
            builder.Dispose();
        }
    }

    public void Append(ref Utf16ValueStringBuilder sb)
    {
        if (_lastCommandDurationMs < _thresholdMs)
        {
            return;
        }

        sb.Append("[yellow]");
        sb.Append(_unformattedString);
        sb.Append("[/]");
    }

    private void AppendNoColors(ref Utf16ValueStringBuilder sb)
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
                    sb.Append(minutes, "N");
                }
                else
                {
                    sb.Append(minutes);
                }

                sb.Append("m ");
                sb.Append(seconds);
                sb.Append('s');
                break;

            case >= 1_000:
                // 59.9s
                double totalSeconds = (_lastCommandDurationMs % 60_000) / 1_000.0;
                sb.Append(totalSeconds, "0.#");
                sb.Append('s');
                break;

            default:
                // 999ms
                sb.Append(_lastCommandDurationMs);
                sb.Append("ms");
                break;
        }
    }
}
