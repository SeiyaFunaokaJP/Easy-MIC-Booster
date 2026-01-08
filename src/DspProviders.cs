using System;
using System.Collections.Generic;
using System.Linq;
using NAudio.Wave;
using NAudio.Dsp;

namespace EasyMICBooster
{
    public enum BandType
    {
        Peaking,
        LowShelf,
        HighShelf
    }

    public class EqBand
    {
        public float Frequency { get; set; }
        public float Gain { get; set; } // in dB
        public float Q { get; set; } = 1.0f; // Bandwidth or Slope
        public BandType Type { get; set; } = BandType.Peaking;
    }

    public class EqualizerSampleProvider : ISampleProvider
    {
        private readonly ISampleProvider _source;
        private readonly object _lock = new object();
        private List<BiQuadFilter> _filters = new List<BiQuadFilter>();
        
        // ISO Standard 1/3 Octave Frequencies (31 Bands)
        public static readonly float[] Frequencies = new float[] { 
            20, 25, 31.5f, 40, 50, 63, 80, 100, 125, 160, 200, 250, 315, 400, 500, 630, 800, 
            1000, 1250, 1600, 2000, 2500, 3150, 4000, 5000, 6300, 8000, 10000, 12500, 16000, 20000 
        };
        
        private int _sampleRate;
        private bool _enabled = true;

        public EqualizerSampleProvider(ISampleProvider source)
        {
            _source = source;
            _sampleRate = source.WaveFormat.SampleRate;
            UpdateBands(new List<EqBand>());
        }

        public WaveFormat WaveFormat => _source.WaveFormat;

        public bool Enabled
        {
            get => _enabled;
            set => _enabled = value;
        }

        // Parametric EQ Update
        public void UpdateBands(List<EqBand> bands)
        {
            if (bands == null) return;

            var newFilters = new List<BiQuadFilter>();
            
            foreach (var band in bands)
            {
                // Skip inactive bands to save CPU/Phase
                if (Math.Abs(band.Gain) < 0.05f) continue;

                try
                {
                    float f = Math.Clamp(band.Frequency, 20, _sampleRate / 2 - 1);
                    float q = band.Q;
                    float g = band.Gain;

                    switch (band.Type)
                    {
                        case BandType.LowShelf:
                            newFilters.Add(BiQuadFilter.LowShelf(_sampleRate, f, 1.0f, g));
                            break;
                        case BandType.HighShelf:
                            newFilters.Add(BiQuadFilter.HighShelf(_sampleRate, f, 1.0f, g));
                            break;
                        case BandType.Peaking:
                        default:
                            newFilters.Add(BiQuadFilter.PeakingEQ(_sampleRate, f, q, g));
                            break;
                    }
                }
                catch { }
            }

            lock (_lock)
            {
                _filters = newFilters;
            }
        }

        public int Read(float[] buffer, int offset, int count)
        {
            int samplesRead = _source.Read(buffer, offset, count);

            if (_enabled && samplesRead > 0)
            {
                lock (_lock)
                {
                    if (_filters.Count > 0)
                    {
                        var filters = _filters; // Local ref
                        for (int i = 0; i < samplesRead; i++)
                        {
                            float sample = buffer[offset + i];
                            foreach (var filter in filters)
                            {
                                sample = filter.Transform(sample);
                            }
                            buffer[offset + i] = sample;
                        }
                    }
                }
            }

            return samplesRead;
        }
    }

    public class MeteringSampleProvider : ISampleProvider
    {
        private readonly ISampleProvider _source;
        public event EventHandler<float[]>? StreamVolume;

        public MeteringSampleProvider(ISampleProvider source)
        {
            _source = source;
        }

        public WaveFormat WaveFormat => _source.WaveFormat;

        public int Read(float[] buffer, int offset, int count)
        {
            int samplesRead = _source.Read(buffer, offset, count);
            
            // Raise event with copy of buffer or direct?
            // Direct is unsafe if event handler modifies it, but faster.
            // Visualization usually just Reads.
            
            if (samplesRead > 0 && StreamVolume != null)
            {
                // Pass buffer slice? 
                // Creating a copy is safer but alloc heavy.
                // Let's rely on handler being read-only.
                float[] chunk = new float[samplesRead];
                Array.Copy(buffer, offset, chunk, 0, samplesRead);
                StreamVolume.Invoke(this, chunk);
            }

            return samplesRead;
        }
    }

    public class NoiseGateSampleProvider : ISampleProvider
    {
        private readonly ISampleProvider _source;
        private float _thresholdDb = -80.0f; // in dB
        
        // Envelope State
        private float _envelope = 0.0f;
        
        // Time constants (in samples)
        private float _attackCoef;
        private float _releaseCoef;
        
        // Defaults
        private const float DefaultAttackMs = 10.0f;
        private const float DefaultReleaseMs = 200.0f;
        
        private int _sampleRate;

        public NoiseGateSampleProvider(ISampleProvider source)
        {
            _source = source;
            _sampleRate = source.WaveFormat.SampleRate;
            SetAttackRelease(DefaultAttackMs, DefaultReleaseMs);
        }

        public WaveFormat WaveFormat => _source.WaveFormat;

        public float Threshold
        {
            get => _thresholdDb;
            set => _thresholdDb = Math.Clamp(value, -80.0f, 0.0f);
        }
        
        private void SetAttackRelease(float attackMs, float releaseMs)
        {
            _attackCoef = (float)Math.Exp(-1.0 / (0.001 * attackMs * _sampleRate));
            _releaseCoef = (float)Math.Exp(-1.0 / (0.001 * releaseMs * _sampleRate));
        }

        public int Read(float[] buffer, int offset, int count)
        {
            int samplesRead = _source.Read(buffer, offset, count);
            
            // Optimization: Log calc is expensive, maybe use approximate or work in linear domain for threshold comparison?
            // But Envelope is usually linear amplitude.
            // Let's convert ThresholdDb to linear threshold.
            float linearThreshold = (float)Math.Pow(10, _thresholdDb / 20.0);
            
            for (int i = 0; i < samplesRead; i++)
            {
                float input = buffer[offset + i];
                float absInput = Math.Abs(input);
                
                // 1. Envelope Follower (Smooth peak detector)
                if (absInput > _envelope)
                    _envelope = _attackCoef * _envelope + (1 - _attackCoef) * absInput;
                else
                    _envelope = _releaseCoef * _envelope + (1 - _releaseCoef) * absInput;
                
                // 2. Gain Calculation
                // If Envelope < Threshold, apply gain reduction
                float gain = 1.0f;
                if (_envelope < linearThreshold)
                {
                    // Expand/Gate
                    // Simple Hard/Soft Knee implementation:
                    // Determine ratio of envelope to threshold
                    // Gain = (Envelope / Threshold) ^ (Ratio - 1) ? 
                    // Simple Noise Gate: just attenuate essentially to zero (or low floor)
                    // But we want smooth transition.
                    // Let's use a smoother curve:
                    // When quite below threshold, gain drops rapidly.
                    
                    // Simple approach:
                    // If env < thresh, gain = (env / thresh) ^ slope
                    // Slope 2.0 = gentle, 10.0 = hard.
                    // Let's try slope 6.0
                    // Start modifying to be more aggressive
                    // Slope: Increase to 12.0 provides a much sharper gate.
                    if (linearThreshold > 1e-6f) 
                        gain = (float)Math.Pow(_envelope / linearThreshold, 12.0); // Sharper curve for effective gating
                }
                
                // 3. Apply Gain
                buffer[offset + i] = input * gain;
            }

            return samplesRead;
        }
    }

    /// <summary>
    /// Soft Limiter: Reduces gain smoothly when signal exceeds threshold.
    /// Uses a simple soft-knee compression algorithm (not hard clipping).
    /// </summary>
    public class LimiterSampleProvider : ISampleProvider
    {
        private readonly ISampleProvider _source;
        private float _thresholdDb = 0.0f; // in dB (0dB = no limiting, -6dB = limit earlier)
        private float _ratio = 10.0f; // High ratio for limiting effect (10:1 to infinity:1)
        
        // Envelope State
        private float _envelope = 0.0f;
        
        // Time constants (in samples)
        private float _attackCoef;
        private float _releaseCoef;
        
        // Defaults
        private const float DefaultAttackMs = 1.0f;   // Fast attack for limiting
        private const float DefaultReleaseMs = 100.0f;
        
        private int _sampleRate;
        private bool _enabled = true;

        public LimiterSampleProvider(ISampleProvider source)
        {
            _source = source;
            _sampleRate = source.WaveFormat.SampleRate;
            SetAttackRelease(DefaultAttackMs, DefaultReleaseMs);
        }

        public WaveFormat WaveFormat => _source.WaveFormat;

        public bool Enabled
        {
            get => _enabled;
            set => _enabled = value;
        }

        public float ThresholdDb
        {
            get => _thresholdDb;
            set => _thresholdDb = Math.Clamp(value, -60.0f, 100.0f);
        }

        public float Ratio
        {
            get => _ratio;
            set => _ratio = Math.Clamp(value, 1.0f, 100.0f);
        }
        
        private void SetAttackRelease(float attackMs, float releaseMs)
        {
            _attackCoef = (float)Math.Exp(-1.0 / (0.001 * attackMs * _sampleRate));
            _releaseCoef = (float)Math.Exp(-1.0 / (0.001 * releaseMs * _sampleRate));
        }

        public int Read(float[] buffer, int offset, int count)
        {
            int samplesRead = _source.Read(buffer, offset, count);
            
            if (!_enabled) // Remove threshold check here, allow limiting at any level
            {
                return samplesRead;
            }
            
            float linearThreshold = (float)Math.Pow(10, _thresholdDb / 20.0);
            
            for (int i = 0; i < samplesRead; i++)
            {
                float input = buffer[offset + i];
                float absInput = Math.Abs(input);
                
                // 1. Envelope Follower (Peak Detector)
                if (absInput > _envelope)
                    _envelope = _attackCoef * _envelope + (1 - _attackCoef) * absInput;
                else
                    _envelope = _releaseCoef * _envelope + (1 - _releaseCoef) * absInput;
                
                // 2. Gain Calculation (Soft Knee Compression/Limiting)
                float gain = 1.0f;
                if (_envelope > linearThreshold)
                {
                    // Over threshold: apply compression
                    // Output Level (dB) = Threshold + (Input - Threshold) / Ratio
                    // Gain = Output / Input
                    float envDb = 20.0f * (float)Math.Log10(Math.Max(_envelope, 1e-10f));
                    float overDb = envDb - _thresholdDb;
                    float targetDb = _thresholdDb + overDb / _ratio;
                    float targetLinear = (float)Math.Pow(10, targetDb / 20.0);
                    gain = targetLinear / Math.Max(_envelope, 1e-10f);
                }
                
                // 3. Apply Gain
                buffer[offset + i] = input * Math.Min(gain, 1.0f); // Never boost
            }

            return samplesRead;
        }
    }
}
