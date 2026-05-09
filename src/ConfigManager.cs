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

    public class DeviceProfile
    {
        public string Name { get; set; } = "Default";
        public string InputDeviceId { get; set; } = "";
        public string OutputDeviceId { get; set; } = "";
        public string InputDeviceName { get; set; } = "";
        public string OutputDeviceName { get; set; } = "";
    }

    public class ConfigManager
    {
        private readonly string _configDir;
        private readonly string _configPath;
        private readonly string _presetsDir;
        private readonly string _deviceProfilesDir;

        public ConfigManager()
        {
            _configDir = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
                "EasyMICBooster");
            _configPath = Path.Combine(_configDir, "config.ini");
            _presetsDir = Path.Combine(_configDir, "Presets");
            _deviceProfilesDir = Path.Combine(_configDir, "DeviceProfiles");

            if (!Directory.Exists(_configDir)) Directory.CreateDirectory(_configDir);
            if (!Directory.Exists(_presetsDir)) Directory.CreateDirectory(_presetsDir);
            if (!Directory.Exists(_deviceProfilesDir)) Directory.CreateDirectory(_deviceProfilesDir);

            if (!File.Exists(_configPath))
            {
                WriteConfig(1.0f, true, "", "", false, new List<EqBand>(), -80.0f, 40.0f, false, false, "", "", "en", true, false);
            }
        }

        public string ConfigDirectory => _configDir;
        public string PresetsDirectory => _presetsDir;
        public string DeviceProfilesDirectory => _deviceProfilesDir;

        public (float gain, bool enabled, string inputId, string outputId, bool unlockLimit, List<EqBand> eqBands, float noiseGateThreshold, float limiterThreshold, bool limiterEnabled, bool flatMode, string lastPresetName, string lastDeviceProfileName, string language, bool updateCheck, bool noiseSuppression) ReadConfig()
        {
            float gain = 1.0f;
            bool enabled = true;
            string inputId = "";
            string outputId = "";
            bool unlockLimit = false;
            List<EqBand> eqBands = new List<EqBand>();
            float noiseGateThreshold = -80.0f;
            float limiterThreshold = 40.0f;
            bool limiterEnabled = false;
            bool flatMode = false;
            string lastPresetName = "";
            string lastDeviceProfileName = "";
            string language = "en";
            bool updateCheck = true;
            bool noiseSuppression = false;

            try
            {
                if (!File.Exists(_configPath))
                {
                    return (gain, enabled, inputId, outputId, unlockLimit, eqBands, noiseGateThreshold, limiterThreshold, limiterEnabled, flatMode, lastPresetName, lastDeviceProfileName, language, updateCheck, noiseSuppression);
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
                    else if (trimmed.StartsWith("LastDeviceProfile="))
                    {
                        lastDeviceProfileName = trimmed.Substring(18);
                    }
                    else if (trimmed.StartsWith("Language="))
                    {
                        language = trimmed.Substring(9);
                    }
                    else if (trimmed.StartsWith("UpdateCheck="))
                    {
                        updateCheck = trimmed.Substring(12) == "1";
                    }
                    else if (trimmed.StartsWith("NoiseSuppression="))
                    {
                        noiseSuppression = trimmed.Substring(17) == "1";
                    }
                }
            }
            catch (Exception) { }

            return (gain, enabled, inputId, outputId, unlockLimit, eqBands, noiseGateThreshold, limiterThreshold, limiterEnabled, flatMode, lastPresetName, lastDeviceProfileName, language, updateCheck, noiseSuppression);
        }

        public void WriteConfig(float gain, bool enabled, string inputId, string outputId, bool unlockLimit, List<EqBand> eqBands, float noiseGateThreshold, float limiterThreshold, bool limiterEnabled, bool flatMode, string lastPresetName, string lastDeviceProfileName, string language, bool updateCheck, bool noiseSuppression)
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
                sb.AppendLine($"LastDeviceProfile={lastDeviceProfileName}");
                sb.AppendLine($"Language={language}");
                sb.AppendLine($"UpdateCheck={( updateCheck ? "1" : "0" )}");
                sb.AppendLine($"NoiseSuppression={( noiseSuppression ? "1" : "0" )}");

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

        // Device Profile Methods
        public List<DeviceProfile> LoadDeviceProfiles()
        {
            var list = new List<DeviceProfile>();
            try
            {
                if (!Directory.Exists(_deviceProfilesDir)) return list;

                var files = Directory.GetFiles(_deviceProfilesDir, "*.json");
                foreach (var f in files)
                {
                    try
                    {
                        var json = File.ReadAllText(f);
                        var p = System.Text.Json.JsonSerializer.Deserialize<DeviceProfile>(json);
                        if (p != null) list.Add(p);
                    }
                    catch { }
                }
            }
            catch { }
            return list;
        }

        public void SaveDeviceProfile(DeviceProfile profile)
        {
            try
            {
                if (!Directory.Exists(_deviceProfilesDir)) Directory.CreateDirectory(_deviceProfilesDir);

                string safeName = SanitizeFileName(profile.Name);
                if (string.IsNullOrWhiteSpace(safeName)) safeName = "Default";

                string path = Path.Combine(_deviceProfilesDir, safeName + ".json");

                var json = System.Text.Json.JsonSerializer.Serialize(profile, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
                File.WriteAllText(path, json);
            }
            catch { }
        }

        public void DeleteDeviceProfile(string name)
        {
            try
            {
                string safeName = SanitizeFileName(name);
                string path = Path.Combine(_deviceProfilesDir, safeName + ".json");
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
