using System;
using System.Linq;
using System.Collections.Generic;
using NAudio.CoreAudioApi;
using NAudio.Wave;
using NAudio.Wave.SampleProviders;
using NAudio.Dsp;

namespace EasyMICBooster
{
    public class AudioEngine : IDisposable
    {
        private WasapiCapture? _capture;
        private WasapiOut? _render;
        private BufferedWaveProvider? _buffer;
        
        // DSP Chain
        private EqualizerSampleProvider? _equalizer;
        private NoiseGateSampleProvider? _noiseGate;
        private LimiterSampleProvider? _limiter;
        private VolumeSampleProvider? _volumeProvider;
        private MeteringSampleProvider? _metering;
        
        public event EventHandler<float>? PeakLevelReceived;
        public event EventHandler<float[]>? FftDataReceived;
        public event EventHandler<string>? ErrorOccurred;

        private float _gain = 1.0f;
        // private List<EqBand> _currentBands = new List<EqBand>(); // Removed
        private float _noiseGateThreshold = 0.0f;
        private float _limiterThreshold = 40.0f; // Default high
        private bool _limiterEnabled = false;
        
        private bool _isRunning;
        
        // FFT State
        private const int FftLength = 2048; // Power of 2
        private Complex[] _fftBuffer = new Complex[FftLength];
        private int _fftPos = 0;
        private float[]? _lastFftResults;

        public bool IsRunning => _isRunning;

        // ... Start/Stop methods unchanged ...

        private void OnStreamVolume(object? sender, float[] samples)
        {
            if (!_isRunning) return;

            // Metering Logic (Output Level Monitor)
            float max = 0;
            
            for (int i = 0; i < samples.Length; i++)
            {
                float abs = Math.Abs(samples[i]);
                if (abs > max) max = abs;
                // FFT buffer fill logic...
                _fftBuffer[_fftPos].X = (float)(samples[i] * FastFourierTransform.HammingWindow(_fftPos, FftLength));
                _fftBuffer[_fftPos].Y = 0;
                _fftPos++;

                if (_fftPos >= FftLength)
                {
                    PerformFft();
                    _fftPos = 0;
                }
            }
            
            PeakLevelReceived?.Invoke(this, Math.Min(1.0f, max));
        }

        public void Start(MMDevice inputDevice, MMDevice outputDevice, float initialGain, float noiseGateThreshold)
        {
            Stop();

            try
            {
                _gain = initialGain;
                _noiseGateThreshold = noiseGateThreshold;

                // 1. Capture setup
                _capture = new WasapiCapture(inputDevice);
                _capture.ShareMode = AudioClientShareMode.Shared;
                
                // 2. Buffer
                _buffer = new BufferedWaveProvider(_capture.WaveFormat);
                _buffer.DiscardOnBufferOverflow = true;
                
                // 3. Build DSP Chain
                ISampleProvider source = _buffer.ToSampleProvider();

                //   a. Equalizer (First in chain)
                _equalizer = new EqualizerSampleProvider(source);
                // Initial update (Flat) is done in constructor

                //   b. Gain (Boost next)
                _volumeProvider = new VolumeSampleProvider(_equalizer);
                _volumeProvider.Volume = _gain;
                
                //   c. Noise Gate
                _noiseGate = new NoiseGateSampleProvider(_volumeProvider);
                _noiseGate.Threshold = _noiseGateThreshold;
                
                //   d. Limiter (Soft limit output level)
                _limiter = new LimiterSampleProvider(_noiseGate);
                _limiter.ThresholdDb = _limiterThreshold;
                _limiter.Enabled = _limiterEnabled;
                
                //   e. Metering (Final Tap for Visualization)
                _metering = new MeteringSampleProvider(_limiter);
                _metering.StreamVolume += OnStreamVolume;
                
                // 4. Output setup
                _render = new WasapiOut(outputDevice, AudioClientShareMode.Shared, true, 50);

                IWaveProvider playbackProvider = _metering.ToWaveProvider16(); // Use metering provider

                _capture.DataAvailable += OnDataAvailable;
                _render.Init(playbackProvider);

                _capture.StartRecording();
                _render.Play();
                
                _isRunning = true;
            }
            catch (Exception ex)
            {
                Stop();
                ErrorOccurred?.Invoke(this, $"オーディオエンジンの起動に失敗しました:\n{ex.Message}");
            }
        }

        public void Stop()
        {
            _isRunning = false;

            if (_metering != null)
            {
                _metering.StreamVolume -= OnStreamVolume;
                _metering = null;
            }

            if (_capture != null)
            {
                _capture.DataAvailable -= OnDataAvailable;
                _capture.StopRecording();
                _capture.Dispose();
                _capture = null;
            }

            if (_render != null)
            {
                _render.Stop();
                _render.Dispose();
                _render = null;
            }

            _buffer = null;
            _equalizer = null;
            _noiseGate = null;
            _limiter = null;
            _volumeProvider = null;
        }

        public void UpdateParametricEq(float volumeDb, List<EqBand> bands)
        {
            // Update Volume
            // Map dB to linear scalar
            // Max limit +40dB (100x) is handled by UI/Config, here we just apply.
            float scalar = (float)Math.Pow(10, volumeDb / 20.0);
            _gain = scalar;
            if (_volumeProvider != null) _volumeProvider.Volume = _gain;

            // Update EQ Filters
            if (_equalizer != null)
            {
                _equalizer.UpdateBands(bands);
            }
        }

        public void SetNoiseGateThreshold(float threshold)
        {
            _noiseGateThreshold = threshold;
            if (_noiseGate != null)
            {
                _noiseGate.Threshold = threshold;
            }
        }

        public void SetLimiterThreshold(float thresholdDb)
        {
            _limiterThreshold = thresholdDb;
            if (_limiter != null)
            {
                _limiter.ThresholdDb = thresholdDb;
            }
        }

        public void SetLimiterEnabled(bool enabled)
        {
            _limiterEnabled = enabled;
            if (_limiter != null)
            {
                _limiter.Enabled = enabled;
            }
        }

        private void OnDataAvailable(object? sender, WaveInEventArgs e)
        {
            if (_buffer == null) return;
            // Just feed buffer
            _buffer.AddSamples(e.Buffer, 0, e.BytesRecorded);
        }



        private void PerformFft()
        {
            // Calculate FFT
            // Since we modified _fftBuffer in place, we can just run it.
            // Note: NAudio's FFT usually requires log2(n) argument
            int m = (int)Math.Log(FftLength, 2.0);
            FastFourierTransform.FFT(true, m, _fftBuffer);

            // Extract Magnitude
            // Only need first half (Nyquist)
            int outputLen = FftLength / 2;
            if (_lastFftResults == null || _lastFftResults.Length != outputLen)
            {
                _lastFftResults = new float[outputLen];
            }

            for (int i = 0; i < outputLen; i++)
            {
                // Magnitude = sqrt(re^2 + im^2)
                double mag = Math.Sqrt(_fftBuffer[i].X * _fftBuffer[i].X + _fftBuffer[i].Y * _fftBuffer[i].Y);
                
                // Convert to dB-like scale for visualization (0.0 to 1.0 range)
                // Typical audio range -90dB to 0dB. 
                // Let's use log scale simply.
                // 20*log10(mag) gives dB.
                
                double db = 20 * Math.Log10(mag);
                
                // Normalize for display: -60dB -> 0.0, 0dB -> 1.0 (clamped)
                double normalized = (db + 60) / 60.0;
                _lastFftResults[i] = (float)Math.Clamp(normalized, 0, 1);
            }

            FftDataReceived?.Invoke(this, _lastFftResults);
        }

        public static IEnumerable<MMDevice> GetInputDevices()
        {
            using var enumerator = new MMDeviceEnumerator();
            return enumerator.EnumerateAudioEndPoints(DataFlow.Capture, DeviceState.Active).ToList();
        }

        public static IEnumerable<MMDevice> GetOutputDevices()
        {
            using var enumerator = new MMDeviceEnumerator();
            return enumerator.EnumerateAudioEndPoints(DataFlow.Render, DeviceState.Active).ToList();
        }

        public void Dispose()
        {
            Stop();
        }
    }
}
