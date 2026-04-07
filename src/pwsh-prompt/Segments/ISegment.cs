using System.Text;

namespace Prompt.Segments;

internal interface ISegment
{
    int UnformattedLength { get; }

    void Append(ref ValueStringBuilder sb);
}
