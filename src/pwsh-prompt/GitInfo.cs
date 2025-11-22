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
    public static StringSegment GetBranchName(string path)
    {
        // Check environment variable cache first
        var cachedBranch = Environment.GetEnvironmentVariable("PROMPT_GIT_BRANCH_CACHED");
        if (!string.IsNullOrEmpty(cachedBranch))
        {
            return cachedBranch;
        }

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
        // For worktrees, gitFolder points to .git/worktrees/<name>, so we need to go up to find config
        string configPath = Path.Combine(gitFolder, "config");

        if (!File.Exists(configPath))
        {
            // Try parent directory for worktrees (.git/worktrees/<name>/../..)
            var parentGitFolder = Path.GetDirectoryName(Path.GetDirectoryName(gitFolder));
            if (parentGitFolder != null)
            {
                configPath = Path.Combine(parentGitFolder, "config");
            }
        }

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

        // Write to environment variable for PowerShell to cache
        Environment.SetEnvironmentVariable("PROMPT_GIT_BRANCH_OUT", branch.ToString());
        Environment.SetEnvironmentVariable("PROMPT_GIT_DIR_OUT", gitFolder);

        return branch;
    }

    public static bool TryFindGitFolder(ReadOnlySpan<char> path, out string gitDirectory)
    {
        // Check environment variable cache first
        var cachedGitDir = Environment.GetEnvironmentVariable("PROMPT_GIT_DIR_CACHED");
        if (!string.IsNullOrEmpty(cachedGitDir))
        {
            gitDirectory = cachedGitDir;
            return true;
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

                gitDirectory = gitPath;
                return true;
            }

            // Check if .git is a file (git worktree)
            if (File.Exists(gitPath))
            {
                string gitFileContent = File.ReadAllText(gitPath).Trim();

                // Git worktree .git file format: "gitdir: /path/to/worktree"
                if (gitFileContent.StartsWith("gitdir: ", StringComparison.Ordinal))
                {
                    string worktreeGitDir = gitFileContent.Substring(8).Trim();

                    if (Directory.Exists(worktreeGitDir))
                    {
                        if (Settings.Debug)
                        {
                            AnsiConsole.MarkupLineInterpolated($"[yellow]Git: found worktree in {worktreeGitDir}[/]");
                        }

                        gitDirectory = worktreeGitDir;
                        return true;
                    }
                }
            }

            if (Settings.Debug)
            {
                AnsiConsole.MarkupLineInterpolated($"[yellow]Git: not found in {gitPath}[/]");
            }

            path = Path.GetDirectoryName(path);

            if (path.Length == 0)
            {
                gitDirectory = string.Empty;
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

        // Use File.ReadLines to avoid loading entire file into memory
        foreach (string line in File.ReadLines(configFile))
        {
            if (line.Length == 0)
            {
                continue;
            }

            if (line[0] == '\t')
            {
                if (currentItem != null)
                {
                    int equalsIndex = line.IndexOf(" = ", StringComparison.Ordinal);
                    if (equalsIndex > 1)
                    {
                        ReadOnlySpan<char> key = line.AsSpan(1, equalsIndex - 1);
                        if (key.SequenceEqual("merge"))
                        {
                            currentItem.Merge = line.Substring(equalsIndex + 3);
                        }
                    }
                }

                continue;
            }

            var match = regex.Match(line);

            if (match.Success)
            {
                currentItem = new ConfigItem
                             {
                                 Type = match.Groups[1].Value,
                                 Name = match.Groups[2].Value
                             };
                yield return currentItem;
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
