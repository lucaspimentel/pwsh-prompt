namespace Prompt;

internal static class Colors
{
    // https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
    // "\x1b" == "\e" == Escape
    public const string Reset = "\e[0m";

    public const string Red = "\x1b[1;31m";
    public const string Green = "\x1b[1;32m";
    public const string Yellow = "\x1b[1;33m";
    public const string Blue = "\x1b[1;34m";
    public const string Magenta = "\x1b[1;35m";
}
