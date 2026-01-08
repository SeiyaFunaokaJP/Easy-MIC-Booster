using System;
using System.IO;
using System.Text;
using System.Collections.Generic;
using System.Globalization;

namespace EasyMICBooster
{
    public class Preset
    {
        public string Name { get; set; } = "Default";
        public List<EqBand> Bands { get; set; } = new List<EqBand>();
        public bool FlatMode { get; set; }
        public double MinFreq { get; set; } = 20;
        public double MaxFreq { get; set; } = 20000;
        public bool UnlockLimit { get; set; } = false;
        public float NoiseGateThreshold { get; set; } = -80.0f;
    }

    public class ConfigManager
    {
        private readonly string _configPath;
        private readonly string _presetsDir;
        private const string DefaultConfigPath = @"C:\Program Files\EasyAPO\config.ini";

        public ConfigManager()
        {
            // Use local config/presets
            _configPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "config.ini");
            _presetsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Presets");
            
            if (!File.Exists(_configPath))
            {
                WriteConfig(1.0f, true, "", "", false, new List<EqBand>(), -80.0f, 40.0f, false, false, "", "en", true);
            }
            if (!Directory.Exists(_presetsDir))
            {
                Directory.CreateDirectory(_presetsDir);
            }
        }

        public (float gain, bool enabled, string inputId, string outputId, bool unlockLimit, List<EqBand> eqBands, float noiseGateThreshold, float limiterThreshold, bool limiterEnabled, bool flatMode, string lastPresetName, string language, bool updateCheck) ReadConfig()
        {
            float gain = 1.0f;
            bool enabled = true; // Default True
            string inputId = "";
            string outputId = "";
            bool unlockLimit = false;
            List<EqBand> eqBands = new List<EqBand>();
            float noiseGateThreshold = -80.0f; // Default low
            float limiterThreshold = 40.0f; // Default high (safe)
            bool limiterEnabled = false; // Default disabled
            bool flatMode = false;
            string lastPresetName = "";
            string language = "en"; // Default English
            bool updateCheck = true; // Default True

            try
            {
                if (!File.Exists(_configPath))
                {
                    return (gain, enabled, inputId, outputId, unlockLimit, eqBands, noiseGateThreshold, limiterThreshold, limiterEnabled, flatMode, lastPresetName, language, updateCheck);
                }

                var lines = File.ReadAllLines(_configPath);
                
                foreach (var line in lines)
                {
                    var trimmed = line.Trim();
                    
                    if (trimmed.StartsWith("Value="))
                    {
                        if (float.TryParse(trimmed.Substring(6), out float val)) gain = val;
                    }
                    else if (trimmed.StartsWith("Enabled="))
                    {
                        enabled = trimmed.Substring(8) == "1";
                    }
                    else if (trimmed.StartsWith("InputDevice="))
                    {
                        inputId = trimmed.Substring(12);
                    }
                    else if (trimmed.StartsWith("OutputDevice="))
                    {
                        outputId = trimmed.Substring(13);
                    }
                    else if (trimmed.StartsWith("UnlockLimit="))
                    {
                         unlockLimit = trimmed.Substring(12) == "1";
                    }
                    else if (trimmed.StartsWith("EqBands="))
                    {
                        var bandStr = trimmed.Substring(8);
                        eqBands = ParseEqBands(bandStr);
                    }
                    else if (trimmed.StartsWith("NoiseGate="))
                    {
                         if (float.TryParse(trimmed.Substring(10), out float val)) 
                         {
                             if (val >= 0) val = -80.0f;
                             noiseGateThreshold = val;
                         }
                    }
                    else if (trimmed.StartsWith("FlatMode="))
                    {
                        flatMode = trimmed.Substring(9) == "1";
                    }
                    else if (trimmed.StartsWith("Limiter="))
                    {
                         if (float.TryParse(trimmed.Substring(8), out float val))
                         {
                             // Just accept value.
                             // Just accept value.
                             limiterThreshold = val;
                         }
                    }
                    else if (trimmed.StartsWith("LimiterEnabled="))
                    {
                        limiterEnabled = trimmed.Substring(15) == "1";
                    }
                    else if (trimmed.StartsWith("LastPreset="))
                    {
                        lastPresetName = trimmed.Substring(11);
                    }
                    else if (trimmed.StartsWith("Language="))
                    {
                        language = trimmed.Substring(9);
                    }
                    else if (trimmed.StartsWith("UpdateCheck="))
                    {
                        updateCheck = trimmed.Substring(12) == "1";
                    }
                }
            }
            catch (Exception) { }

            return (gain, enabled, inputId, outputId, unlockLimit, eqBands, noiseGateThreshold, limiterThreshold, limiterEnabled, flatMode, lastPresetName, language, updateCheck);
        }

        public void WriteConfig(float gain, bool enabled, string inputId, string outputId, bool unlockLimit, List<EqBand> eqBands, float noiseGateThreshold, float limiterThreshold, bool limiterEnabled, bool flatMode, string lastPresetName, string language, bool updateCheck)
        {
            try
            {
                var sb = new StringBuilder();
                sb.AppendLine("[Settings]");
                sb.AppendLine($"Value={gain:F1}");
                sb.AppendLine($"Enabled={( enabled ? "1" : "0" )}");
                sb.AppendLine($"InputDevice={inputId}");
                sb.AppendLine($"OutputDevice={outputId}");
                sb.AppendLine($"UnlockLimit={( unlockLimit ? "1" : "0" )}");
                sb.AppendLine($"EqBands={SerializeEqBands(eqBands)}");
                sb.AppendLine($"NoiseGate={noiseGateThreshold:F2}");
                sb.AppendLine($"Limiter={limiterThreshold:F1}");
                sb.AppendLine($"LimiterEnabled={( limiterEnabled ? "1" : "0" )}");
                sb.AppendLine($"FlatMode={( flatMode ? "1" : "0" )}");
                sb.AppendLine($"LastPreset={lastPresetName}");
                sb.AppendLine($"Language={language}");
                sb.AppendLine($"UpdateCheck={( updateCheck ? "1" : "0" )}");

                File.WriteAllText(_configPath, sb.ToString());
            }
            catch (Exception) { }
        }
        
        // Preset Methods
        public List<Preset> LoadPresets()
        {
            var list = new List<Preset>();
            try
            {
                if (!Directory.Exists(_presetsDir)) return list;
                
                var files = Directory.GetFiles(_presetsDir, "*.json");
                foreach (var f in files)
                {
                    try
                    {
                        var json = File.ReadAllText(f);
                        var p = System.Text.Json.JsonSerializer.Deserialize<Preset>(json);
                        if (p != null) list.Add(p);
                    }
                    catch { }
                }
            }
            catch { }
            return list;
        }

        public void SavePreset(Preset preset)
        {
            try
            {
                if (!Directory.Exists(_presetsDir)) Directory.CreateDirectory(_presetsDir);
                
                string safeName = SanitizeFileName(preset.Name);
                if (string.IsNullOrWhiteSpace(safeName)) safeName = "Default";
                
                string path = Path.Combine(_presetsDir, safeName + ".json");
                
                var json = System.Text.Json.JsonSerializer.Serialize(preset, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(path, json);
            }
            catch { }
        }
        
        public void DeletePreset(string name)
        {
            try
            {
                string safeName = SanitizeFileName(name);
                string path = Path.Combine(_presetsDir, safeName + ".json");
                if (File.Exists(path)) File.Delete(path);
            }
            catch { }
        }
        
        private string SanitizeFileName(string name)
        {
            var invalid = Path.GetInvalidFileNameChars();
            foreach (var c in invalid) name = name.Replace(c, '_');
            return name;
        }

        private List<EqBand> ParseEqBands(string data)
        {
            var list = new List<EqBand>();
            if (string.IsNullOrWhiteSpace(data)) return list;

            var items = data.Split(',', StringSplitOptions.RemoveEmptyEntries);
            foreach (var item in items)
            {
                var parts = item.Split(':');
                if (parts.Length == 3)
                {
                    if (float.TryParse(parts[0], NumberStyles.Any, CultureInfo.InvariantCulture, out float f) &&
                        float.TryParse(parts[1], NumberStyles.Any, CultureInfo.InvariantCulture, out float g) &&
                        float.TryParse(parts[2], NumberStyles.Any, CultureInfo.InvariantCulture, out float q))
                    {
                        list.Add(new EqBand { Frequency = f, Gain = g, Q = q });
                    }
                }
            }
            return list;
        }

        private string SerializeEqBands(List<EqBand> bands)
        {
            var sb = new StringBuilder();
            foreach (var band in bands)
            {
                if (sb.Length > 0) sb.Append(",");
                sb.Append($"{band.Frequency.ToString(CultureInfo.InvariantCulture)}:{band.Gain.ToString(CultureInfo.InvariantCulture)}:{band.Q.ToString(CultureInfo.InvariantCulture)}");
            }
            return sb.ToString();
        }
    }
}
