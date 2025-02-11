// <copyright file="GitInfo.cs" company="Datadog">
// Unless explicitly stated otherwise all files in this repository are licensed under the Apache 2 License.
// This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2017 Datadog, Inc.
// </copyright>

using System;
using System.Collections.Generic;
using System.IO;
using System.Text.RegularExpressions;
using Microsoft.Extensions.Primitives;
using Spectre.Console;

namespace Prompt;

internal static partial class GitInfo
{
    private static string? _gitDirectory;
    private static bool? _foundGitDirectory;

    public static StringSegment GetBranchName(string path)
    {
        if (!TryFindGitFolder(path, out var gitFolder))
        {
            return StringSegment.Empty;
        }

        StringSegment branch = StringSegment.Empty;

        // Get Git commit
        string headPath = Path.Combine(gitFolder, "HEAD");

        if (File.Exists(headPath))
        {
            string head = File.ReadAllText(headPath).Trim();

            // Symbolic Reference
            if (head.StartsWith("ref:", StringComparison.Ordinal))
            {
                branch = new StringSegment(head, 4, head.Length - 4);
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

        if (branch.Length > 0)
        {
            branch = branch.Trim();
        }

        if (branch.StartsWith("refs/heads/", StringComparison.Ordinal))
        {
            branch = branch.Subsegment(11);
        }

        return branch;
    }

    public static bool TryFindGitFolder(ReadOnlySpan<char> path, out string gitDirectory)
    {
        if (_foundGitDirectory is { } found && _gitDirectory is not null)
        {
            gitDirectory = _gitDirectory;
            return found;
        }

        if (Settings.Debug)
        {
            AnsiConsole.WriteLine();
        }

        while (true)
        {
            string gitPath = Path.Join(path, ".git");

            if (Directory.Exists(gitPath))
            {
                if (Settings.Debug)
                {
                    AnsiConsole.MarkupLineInterpolated($"[yellow]Git: found in {gitPath}[/]");
                }

                _gitDirectory = gitDirectory = gitPath;
                _foundGitDirectory = true;
                return true;
            }

            if (Settings.Debug)
            {
                AnsiConsole.MarkupLineInterpolated($"[yellow]Git: not found in {gitPath}[/]");
            }

            path = Path.GetDirectoryName(path);

            if (path.Length == 0)
            {
                _gitDirectory = gitDirectory = string.Empty;
                _foundGitDirectory = false;
                return false;
            }
        }
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
                    string[] keyValue = line[1..].Split(" = ", StringSplitOptions.RemoveEmptyEntries);

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
