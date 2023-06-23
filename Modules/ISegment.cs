using Cysharp.Text;

namespace Prompt.Modules;

internal interface ISegment
{
    int UnformattedLength { get; }

    void Append(ref Utf16ValueStringBuilder sb);
}

internal abstract class Segment : ISegment
{
    public abstract int UnformattedLength { get; }

    public abstract void Append(ref Utf16ValueStringBuilder sb);

    public override string ToString()
    {
        var builder = ZString.CreateStringBuilder();

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
