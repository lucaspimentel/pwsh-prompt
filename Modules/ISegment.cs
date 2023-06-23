using System;
using System.Text;

namespace Prompt.Modules;

internal interface ISegment
{
    int UnformattedLength { get; }

    void Append(ref ValueStringBuilder sb);
}
