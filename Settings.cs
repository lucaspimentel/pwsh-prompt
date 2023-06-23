using System;

namespace Prompt;

internal static class Settings
{
    public static readonly bool Debug = Environment.GetEnvironmentVariable("DEBUG_PROMPT") == "1";

    public const int LastCommandDurationThresholdMs = 30;

    public const string Prompt = "❯";
}
