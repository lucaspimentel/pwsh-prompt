// <copyright file="GitInfo.cs" company="Datadog">
// Unless explicitly stated otherwise all files in this repository are licensed under the Apache 2 License.
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2017 Datadog, Inc.
// </copyright>

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;

namespace Prompt;

internal static partial class GitInfo
{
    public static ReadOnlySpan<char> GetBranchNameFrom(string path)
    {
        ReadOnlySpan<char> branch = default;

        // Get Git commit
        string headPath = Path.Combine(path, "HEAD");

        if (File.Exists(headPath))
        {
            string head = File.ReadAllText(headPath).Trim();

            // Symbolic Reference
            if (head.StartsWith("ref:", StringComparison.Ordinal))
            {
                branch = head.AsSpan(4);
            }
        }

        // Process Git Config
        string configPath = Path.Combine(path, "config");

        foreach (var configItem in GetConfigItems(configPath))
        {
            if (configItem.Type == "branch" && branch.Equals(configItem.Merge, StringComparison.Ordinal))
            {
                branch = configItem.Name ?? "";
                break;
            }
        }

        branch = branch.Trim();

        if(branch.StartsWith("refs/heads/", StringComparison.Ordinal))
        {
            branch = branch[11..];
        }

        return branch;
    }

    private static IEnumerable<ConfigItem> GetConfigItems(string configFile)
    {
        if (!File.Exists(configFile))
        {
            return Array.Empty<ConfigItem>();
        }

        return GetConfigItemsIterator(configFile);
    }

    private static IEnumerable<ConfigItem> GetConfigItemsIterator(string configFile)
    {
        ConfigItem? currentItem = null;

        var regex = GitConfigRegex();
        string[] lines = File.ReadAllLines(configFile);

        foreach (string line in lines)
        {
            if (line[0] == '\t')
            {
                if (currentItem != null)
                {
                    string[] keyValue = line[1..].Split(new[] { " = " }, StringSplitOptions.RemoveEmptyEntries);

                    if (keyValue[0] == "merge")
                    {
                        currentItem.Merge = keyValue[1];
                    }
                }

                continue;
            }

            var match = regex.Match(line);

            if (match.Success)
            {
                yield return new ConfigItem
                             {
                                 Type = match.Groups[1].Value,
                                 Name = match.Groups[2].Value
                             };
            }
        }
    }

    internal class ConfigItem
    {
        public string? Type { get; init; }

        public string? Name { get; init; }

        public string? Merge { get; set; }
    }

    [GeneratedRegex("^\\[(.*) \\\"(.*)\\\"\\]")]
    private static partial Regex GitConfigRegex();
}
