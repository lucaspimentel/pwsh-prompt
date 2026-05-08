using System.Diagnostics;

namespace Prompt;

internal static class SegmentTimer
{
    public static readonly Dictionary<string, double>? Timings =
        Settings.Debug ? new Dictionary<string, double>() : null;

    public static long Start() => Settings.Debug ? Stopwatch.GetTimestamp() : 0L;

    public static void Record(string name, long startTimestamp)
    {
        if (!Settings.Debug)
        {
            return;
        }

        Timings![name] = Stopwatch.GetElapsedTime(startTimestamp).TotalMilliseconds;
    }
}
