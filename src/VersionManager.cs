using System;
using System.IO;

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
        /// GitHub raw URL for version checking (update with your repository)
        /// </summary>
        public const string GitHubVersionUrl = "https://raw.githubusercontent.com/SeiyaFunaokaJP/Easy-MIC-Booster/main/version.txt";
        
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
                
                var response = await client.GetAsync(GitHubVersionUrl);
                
                if (!response.IsSuccessStatusCode)
                {
                    return (false, CurrentVersion, $"HTTP {(int)response.StatusCode}");
                }
                
                string latestVersion = (await response.Content.ReadAsStringAsync()).Trim();
                
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
