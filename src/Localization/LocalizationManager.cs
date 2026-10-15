using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Globalization;
using System.IO;
using System.Runtime.CompilerServices;
using System.Text.Json;
using System.Windows;

namespace EasyMICBooster.Localization
{
    public class LocalizationManager : INotifyPropertyChanged
    {
        private static LocalizationManager? _instance;
        public static LocalizationManager Instance => _instance ??= new LocalizationManager();

        public const string FallbackLanguage = "en";

        // Languages shipped in Lang\*.json, in the order shown in the language selector.
        public static readonly (string Code, string NativeName)[] SupportedLanguages =
        {
            ("ja", "日本語"),
            ("en", "English"),
            ("zh-CN", "简体中文"),
            ("zh-TW", "繁體中文"),
            ("ko", "한국어"),
            ("de", "Deutsch"),
            ("fr", "Français"),
            ("es", "Español"),
        };

        private Dictionary<string, string> _translations = new Dictionary<string, string>();
        private Dictionary<string, string> _fallback = new Dictionary<string, string>();
        private string _currentLanguage = "ja";

        public string CurrentLanguage
        {
            get => _currentLanguage;
            private set
            {
                if (_currentLanguage != value)
                {
                    _currentLanguage = value;
                    OnPropertyChanged();
                    OnPropertyChanged("Item[]"); // Notify all indexer bindings
                }
            }
        }

        // Indexer for Binding: {Binding Source={x:Static loc:LocalizationManager.Instance}, Path=[Key]}
        public string this[string key] => GetString(key);

        public event PropertyChangedEventHandler? PropertyChanged;
        protected void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        // Picks the supported language matching the Windows UI language (used on first run).
        public static string DetectSystemLanguage()
        {
            var culture = CultureInfo.CurrentUICulture;
            if (culture.TwoLetterISOLanguageName == "zh")
            {
                // zh-TW / zh-HK / zh-MO / zh-Hant use Traditional Chinese
                string name = culture.Name;
                bool traditional = name.Contains("Hant") || name.EndsWith("-TW") || name.EndsWith("-HK") || name.EndsWith("-MO");
                return traditional ? "zh-TW" : "zh-CN";
            }
            foreach (var (code, _) in SupportedLanguages)
            {
                if (code == culture.TwoLetterISOLanguageName) return code;
            }
            return FallbackLanguage;
        }

        // Loads Lang\<cultureCode>.json. A language this build lacks (e.g. a config
        // written by a newer version) shows in English.
        public void LoadLanguage(string cultureCode)
        {
            if (_fallback.Count == 0) _fallback = ReadLanguageFile(FallbackLanguage) ?? new Dictionary<string, string>();

            var dict = ReadLanguageFile(cultureCode);
            if (dict == null && cultureCode != FallbackLanguage)
            {
                cultureCode = FallbackLanguage;
                dict = _fallback;
            }
            if (dict == null) return;

            _translations = dict;
            CurrentLanguage = cultureCode;

            // Force refresh even if language code didn't change
            // This fixes the issue where initial load of "ja" doesn't update bindings
            // because _currentLanguage was already "ja".
            OnPropertyChanged("Item[]");
        }

        private static Dictionary<string, string>? ReadLanguageFile(string cultureCode)
        {
            try
            {
                // Try BaseDirectory
                string path = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Lang", $"{cultureCode}.json");

                // Fallback: Try CurrentDirectory if different
                if (!File.Exists(path))
                {
                    string currentDir = Directory.GetCurrentDirectory();
                    if (currentDir != AppDomain.CurrentDomain.BaseDirectory)
                    {
                        string altPath = Path.Combine(currentDir, "Lang", $"{cultureCode}.json");
                        if (File.Exists(altPath)) path = altPath;
                    }
                }

                if (!File.Exists(path))
                {
                    System.Diagnostics.Debug.WriteLine($"Language file not found: {path}");
                    return null;
                }

                string json = File.ReadAllText(path);
                return JsonSerializer.Deserialize<Dictionary<string, string>?>(json);
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error loading language {cultureCode}: {ex.Message}");
                return null;
            }
        }

        // Keys missing from a translation fall back to English, then to the key itself.
        public string GetString(string key)
        {
            if (_translations.TryGetValue(key, out string? value)) return value;
            if (_fallback.TryGetValue(key, out value)) return value;
            return key;
        }
    }
}
