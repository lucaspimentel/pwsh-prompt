using System;
using System.Text;

namespace Prompt.Modules;

internal interface ISegment
{
    int UnformattedLength { get; }

    void Append(ref ValueStringBuilder sb);

    string? ToString()
    {
        Span<char> buffer = stackalloc char[128];
        var builder = new ValueStringBuilder(buffer);

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
