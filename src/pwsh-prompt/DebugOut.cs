namespace Prompt;

// DEBUG_PROMPT emission: raw console writes with manual ANSI styling.
// Bypasses AnsiConsole so debug lines are neither wrapped at the profile
// width nor markup-escaped, and the Rust port reproduces the bytes with a
// trivial implementation.
internal static class DebugOut
{
    private const string Yellow = "\x1b[38;5;11m";
    private const string Grey = "\x1b[38;5;8m";
    private const string Reset = "\x1b[0m";

    public static void WriteLine() => Console.Out.Write(Environment.NewLine);

    public static void PlainLine(string text) => Console.Out.Write(text + Environment.NewLine);

    public static void YellowLine(string text) => Console.Out.Write(Yellow + text + Reset + Environment.NewLine);

    public static void WritePlain(string text) => Console.Out.Write(text);

    public static void WriteYellow(string text) => Console.Out.Write(Yellow + text + Reset);

    public static void WriteGrey(string text) => Console.Out.Write(Grey + text + Reset);
}
