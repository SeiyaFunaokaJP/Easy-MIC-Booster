using System;
using System.IO;
using System.Text.Json;

namespace EasyMICBooster
{
    public static class VersionManager
    {
        private static string? _cachedVersion;
        
        /// <summary>
        /// Gets the application version from version.txt
        /// </summary>
        public static string CurrentVersion
        {
            get
            {
                if (_cachedVersion != null) return _cachedVersion;
                
                try
                {
                    string versionPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "version.txt");
                    if (File.Exists(versionPath))
                    {
                        _cachedVersion = File.ReadAllText(versionPath).Trim();
                    }
                    else
                    {
                        // Fallback: Try to read from source directory (for development)
                        string devPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "version.txt");
                        if (File.Exists(devPath))
                        {
                            _cachedVersion = File.ReadAllText(devPath).Trim();
                        }
                        else
                        {
                            _cachedVersion = "0.0.0";
                        }
                    }
                }
                catch
                {
                    _cachedVersion = "0.0.0";
                }
                
                return _cachedVersion;
            }
        }
        
        /// <summary>
        /// Gets the formatted version string for display (e.g., "v1.0.0")
        /// </summary>
        public static string DisplayVersion => $"v{CurrentVersion}";
        
        /// <summary>
        /// GitHub Releases API URL for the latest published release
        /// </summary>
        public const string GitHubLatestReleaseUrl = "https://api.github.com/repos/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest";

        /// <summary>
        /// Checks if a newer version is available on GitHub
        /// </summary>
        /// <returns>Tuple of (isUpdateAvailable, latestVersion, errorMessage)</returns>
        public static async Task<(bool isUpdateAvailable, string latestVersion, string? error)> CheckForUpdateAsync()
        {
            try
            {
                using var client = new System.Net.Http.HttpClient();
                client.Timeout = TimeSpan.FromSeconds(5);
                client.DefaultRequestHeaders.Add("User-Agent", "EasyMICBooster");
                client.DefaultRequestHeaders.Add("Accept", "application/vnd.github+json");

                var response = await client.GetAsync(GitHubLatestReleaseUrl);

                if (!response.IsSuccessStatusCode)
                {
                    return (false, CurrentVersion, $"HTTP {(int)response.StatusCode}");
                }

                using var json = JsonDocument.Parse(await response.Content.ReadAsStringAsync());

                if (!json.RootElement.TryGetProperty("tag_name", out var tagElement))
                {
                    return (false, CurrentVersion, "Invalid version format");
                }

                // Tags are published as "v1.0.5" — strip the prefix for comparison
                string latestVersion = (tagElement.GetString() ?? "").Trim().TrimStart('v', 'V');

                if (string.IsNullOrWhiteSpace(latestVersion) || !latestVersion.Contains('.'))
                {
                    return (false, CurrentVersion, "Invalid version format");
                }

                bool isNewer = CompareVersions(latestVersion, CurrentVersion) > 0;
                return (isNewer, latestVersion, null);
            }
            catch (TaskCanceledException)
            {
                return (false, CurrentVersion, "Timeout");
            }
            catch (System.Net.Http.HttpRequestException)
            {
                return (false, CurrentVersion, "Network error");
            }
            catch (Exception)
            {
                return (false, CurrentVersion, "Unknown error");
            }
        }
        
        /// <summary>
        /// Compares two version strings (e.g., "1.2.0" vs "1.1.0")
        /// Returns positive if v1 > v2, negative if v1 < v2, zero if equal
        /// </summary>
        private static int CompareVersions(string v1, string v2)
        {
            var parts1 = v1.Split('.').Select(int.Parse).ToArray();
            var parts2 = v2.Split('.').Select(int.Parse).ToArray();
            
            int maxLength = Math.Max(parts1.Length, parts2.Length);
            
            for (int i = 0; i < maxLength; i++)
            {
                int p1 = i < parts1.Length ? parts1[i] : 0;
                int p2 = i < parts2.Length ? parts2[i] : 0;
                
                if (p1 != p2) return p1 - p2;
            }
            
            return 0;
        }
    }
}
