using Cysharp.Text;

namespace Prompt.Modules;

internal interface ISegment
{
    int UnformattedLength { get; }

    void Append(ref Utf16ValueStringBuilder sb);

    string? ToString()
    {
        var builder = ZString.CreateStringBuilder(notNested: false);

        try
        {
            Append(ref builder);
            return builder.ToString();
        }
        finally
        {
            builder.Dispose();
        }
    }
}
