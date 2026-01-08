using System;
using System.Collections.Generic;
using System.ComponentModel;
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

        private Dictionary<string, string> _translations = new Dictionary<string, string>();
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

        public void LoadLanguage(string cultureCode)
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

                // Fallback for Development (Input dir) checking? 
                // No, sticking to output is safer.

                if (File.Exists(path))
                {
                    string json = File.ReadAllText(path);
                    var dict = JsonSerializer.Deserialize<Dictionary<string, string>?>(json);
                    if (dict != null)
                    {
                        _translations = dict;
                        CurrentLanguage = cultureCode;
                        
                        // Force refresh even if language code didn't change
                        // This fixes the issue where initial load of "ja" doesn't update bindings
                        // because _currentLanguage was already "ja".
                        OnPropertyChanged("Item[]");
                    }
                }
                else
                {
                    // DEBUG: If file missing, maybe warn?
                    System.Diagnostics.Debug.WriteLine($"Language file not found: {path}");
                    // Don't show MessageBox here to avoid spam loop, but maybe necessary if user is confused?
                }
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error loading language {cultureCode}: {ex.Message}");
            }
        }

        public string GetString(string key)
        {
            return _translations.TryGetValue(key, out string? value) ? value : key;
        }
    }
}
