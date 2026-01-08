using System;
using System.Linq;
using System.Collections.Generic;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Shapes;
using Microsoft.Win32;
using System.Diagnostics;
using NAudio.CoreAudioApi;
using NAudio.Dsp;

namespace EasyMICBooster
{
    public partial class MainWindow : Window
    {
        private readonly ConfigManager _configManager;
        private readonly AudioEngine _audioEngine;
        private bool _isInitializing = true;
        
        // EQ UI State
        private class UiEqPoint 
        { 
            public double Freq; 
            public double Gain; 
            public float Q = 1.0f;
            public BandType Type = BandType.Peaking;
            public Ellipse Visual = new Ellipse(); 
        }
        
        private List<UiEqPoint> _uiPoints = new List<UiEqPoint>();
        private UiEqPoint? _draggingPoint = null;
        private List<Preset> _presets = new List<Preset>();
        private bool _isLoadingPreset = false;
        
        // View Settings
        private double _minFreq = 20;
        private double _maxFreq = 20000;
        private double _currentMaxDb = 40; 
        private bool _hasCheckedUpdate = false;
        
        // Canvas Layout
        private const double ScreenPaddingX = 30; // Left/Right Padding
        private const double ScreenPaddingY = 20; // Top/Bottom Padding
        
        // Spectrum
        private float[]? _currentSpectrum;
        private DateTime _lastSpectrumUpdate = DateTime.MinValue;

        public MainWindow()
        {
            _isInitializing = true;
            InitializeComponent();
            
            // Set window title with version
            this.Title = $"Easy MIC Booster {VersionManager.DisplayVersion}";
            
            _configManager = new ConfigManager();
            _audioEngine = new AudioEngine();
            _audioEngine.PeakLevelReceived += OnPeakLevelReceived;
            _audioEngine.FftDataReceived += OnFftDataReceived;
            _audioEngine.ErrorOccurred += OnAudioError;
            
            LoadDevices();
            LoadSettings();
            _isInitializing = false;
            
            UpdateEqStartEndPoints(); // Ensure initial points
            RefreshAllVisuals();

            UpdateDeviceValidation();
            StartAudio();
        }

        private void LoadDevices()
        {
            try
            {
                var inputDevices = AudioEngine.GetInputDevices().ToList();
                InputDeviceCombo.ItemsSource = inputDevices;
                var outputDevices = AudioEngine.GetOutputDevices().ToList();
                OutputDeviceCombo.ItemsSource = outputDevices;
                // Devices loaded
            }
            catch (Exception ex) { MessageBox.Show($"デバイス読み込みエラー: {ex.Message}"); }
        }

        private void LoadSettings()
        {
            var (gain, enabled, inputId, outputId, unlockLimit, bands, noiseGate, limiterThreshold, limiterEnabled, flatMode, lastPresetName, language, updateCheck) = _configManager.ReadConfig();

            // Enforce Defaults if config is empty logic can be handled here if needed, 
            // but config manager already provides defaults.
            // User requested default: Min20, Max20k, FlatMode, 0dB, Limit OFF, Gate -80.
            // ConfigManager defaults might differ, let's respect what's read but ensure "Always ON".

            _currentMaxDb = unlockLimit ? 100 : 50;
            if (NoiseGateSlider != null) 
            {
                NoiseGateSlider.Minimum = -_currentMaxDb;
                NoiseGateSlider.Maximum = _currentMaxDb; // User Request: Change Max too
            }
            // UI Init
            FlatModeCheck.IsChecked = flatMode;
            UnlockLimitCheck.IsChecked = unlockLimit;
            if (NoiseGateSlider != null) NoiseGateSlider.Value = noiseGate;
            if (NoiseGateValueText != null) NoiseGateValueText.Text = $"{(noiseGate > 0 ? "+" : "")}{noiseGate:F0}dB";
            
            // Limiter - Enabled by default to prevent clipping
            // Users can enable if needed for clipping protection
            _audioEngine?.SetLimiterThreshold(-0.1f); // Safe limit
            if (_audioEngine != null) 
            {
                _audioEngine.SetLimiterEnabled(true);
            }
            
            // Boost Toggle
            if (BoostToggle != null) BoostToggle.IsChecked = enabled;
            
            // Startup Check
            StartupCheckbox.IsChecked = IsStartupEnabled();
            
            // Update Check always enabled, no UI toggle needed
            
            // Freq from config? Config doesn't store Freq Range yet, defaults to 20-20k.
            // If we want to store Freq Range in Config, we need to add it to ConfigManager.
            // For now, default is 20-20k (UI default).

            // Clear old visuals
            var toRemove = EqCanvas.Children.OfType<Ellipse>().ToList();
            foreach (var el in toRemove) EqCanvas.Children.Remove(el);
            _uiPoints.Clear();

            if (bands.Count == 0)
            {
                // Default Flat at 0dB
                AddPointToCanvas(20, 0, 1f, BandType.LowShelf);
                AddPointToCanvas(20000, 0, 1f, BandType.HighShelf);
            }
            else
            {
                foreach (var band in bands)
                {
                    AddPointToCanvas(band.Frequency, band.Gain, 0.1f, BandType.Peaking);
                }
            }
            
            UpdateEqStartEndPoints(); // Ensure 20Hz/20kHz exist and sorted

            // Devices
            if (!string.IsNullOrEmpty(inputId)) 
            { 
                var devices = InputDeviceCombo.ItemsSource as IEnumerable<MMDevice>;
                if (devices != null)
                {
                    var d = devices.FirstOrDefault(x => x.ID == inputId); 
                    if (d != null) InputDeviceCombo.SelectedItem = d; 
                }
            }
            if (!string.IsNullOrEmpty(outputId)) 
            { 
                var devices = OutputDeviceCombo.ItemsSource as IEnumerable<MMDevice>;
                if (devices != null)
                {
                    var d = devices.FirstOrDefault(x => x.ID == outputId); 
                    if (d != null) OutputDeviceCombo.SelectedItem = d; 
                }
            }

            // Load Presets
            _presets = _configManager.LoadPresets();
            UpdatePresetCombo();
            
            // Load Language
            // Initialize Localization
            Localization.LocalizationManager.Instance.LoadLanguage(language);
            
            // Update Language Combo UI
            foreach (ComboBoxItem item in LanguageCombo.Items)
            {
                if (item.Tag?.ToString() == language)
                {
                    LanguageCombo.SelectedItem = item;
                    break;
                }
            }
            
            // Restore Last Preset Name (Text only? or Select?)
            if (!string.IsNullOrEmpty(lastPresetName))
            {
                var p = _presets.FirstOrDefault(x => x.Name == lastPresetName);
                if (p != null)
                {
                    PresetCombo.SelectedItem = p;
                    // ApplyPreset(p); // SelectionChanged will trigger ApplyPreset
                }
                else
                {
                    // Preset might be deleted, just show text?
                    // PresetCombo.Text = lastPresetName; 
                }
            }
        }

        private void UpdatePresetCombo()
        {
            _isLoadingPreset = true; 
            PresetCombo.ItemsSource = null;
            PresetCombo.ItemsSource = _presets;
            _isLoadingPreset = false;
        }
        
        private void RefreshAllVisuals()
        {
            UpdateEqStartEndPoints();
            DrawGridLines();
            UpdatePointVisuals();
            UpdateEqCurve();
            UpdateNoiseGateVisual();
        }

        // Ensure Shelf Points exist AND are within view range?
        // Actually, shelf points are logically fixed at 'Edges'. If user changes view range to 100-1000Hz,
        // the 20Hz shelf point should still function but might be off-screen.
        // User requested: "Points at interaction points". 
        // Let's keep logic: Leftmost point is LowShelf, Rightmost is HighShelf.
        private void UpdateEqStartEndPoints()
        {
            // Just ensure we have points near current min/max freq?
            // Or stick to 20/20k absolute? Stick to absolute 20Hz/20kHz for now as they are standard logic.
            // If they are off-screen, user can't drag them.
            // If user changes range, maybe we should move these points or add new ones?
            // For now, let's just make sure 20Hz and 20kHz exist.
            
            // Ensure 20Hz LowShelf exists and snap if drifted (prevents duplicates)
            var lowShelf = _uiPoints.FirstOrDefault(p => p.Type == BandType.LowShelf);
            if (lowShelf != null)
            {
                lowShelf.Freq = 20; 
            }
            else if (!_uiPoints.Any(p => Math.Abs(p.Freq - 20) < 1.0))
            {
                AddPointToCanvas(20, 0, 0.7f, BandType.LowShelf);
            }

            // Ensure 20kHz HighShelf exists and snap if drifted
            var highShelf = _uiPoints.FirstOrDefault(p => p.Type == BandType.HighShelf);
            if (highShelf != null)
            {
                highShelf.Freq = 20000;
            }
            else if (!_uiPoints.Any(p => Math.Abs(p.Freq - 20000) < 1.0))
            {
                AddPointToCanvas(20000, 0, 0.7f, BandType.HighShelf);
            }
                
            _uiPoints = _uiPoints.OrderBy(p => p.Freq).ToList();
        }
        
        private void FreqRange_Changed(object sender, RoutedEventArgs e)
        {
            if (_isInitializing) return;
            if (double.TryParse(MinFreqInput.Text, out double min)) _minFreq = min;
            if (double.TryParse(MaxFreqInput.Text, out double max)) _maxFreq = max;
            
            // Constrain
            if (_minFreq < 20) _minFreq = 20;
            if (_maxFreq > 24000) _maxFreq = 24000;
            if (_minFreq >= _maxFreq) _minFreq = _maxFreq - 100;
            
            RefreshAllVisuals();
            UpdateAudioEq(); 
        }



        private void DrawGridLines()
        {
            EqGridCanvas.Children.Clear();
            double w = EqCanvas.ActualWidth > 0 ? EqCanvas.ActualWidth : 450;
            double h = EqCanvas.ActualHeight > 0 ? EqCanvas.ActualHeight : 240;

            // X-Axis Grid & Labels (Log Scale)
            // Major: 100, 1k, 10k
            double[] majors = { 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000 };
            
            foreach (var f in majors)
            {
                if (f < _minFreq || f > _maxFreq) continue;
                
                double x = FreqToX(f, w);
                
                // Line
                EqGridCanvas.Children.Add(new Line { X1 = x, Y1 = ScreenPaddingY, X2 = x, Y2 = h - ScreenPaddingY, Stroke = Brushes.Gray, Opacity = 0.2 });
                
                // Label
                var tb = new TextBlock
                {
                    Text = f >= 1000 ? (f/1000) + "k" : f.ToString(),
                    FontSize = 10, Foreground = Brushes.Gray,
                    HorizontalAlignment = HorizontalAlignment.Center
                };
                Canvas.SetLeft(tb, x - 10);
                Canvas.SetTop(tb, h - ScreenPaddingY + 2);
                EqGridCanvas.Children.Add(tb);
            }
            
            // Y-Axis Grid & Labels
            // Step size depends on range
            double step = _currentMaxDb >= 40 ? 10 : 5;
            
            for (double db = 0; db <= _currentMaxDb; db += step)
            {
                // Positive
                DrawHorizontalLineAndLabel(db, w, h);
                if (db != 0) DrawHorizontalLineAndLabel(-db, w, h);
            }
            
            // Adjust Zero Line in Grid
            double zeroY = DbToY(0, h);
            if (ZeroDbLine != null)
            {
                ZeroDbLine.X1 = ScreenPaddingX; ZeroDbLine.X2 = w - ScreenPaddingX;
                ZeroDbLine.Y1 = zeroY; ZeroDbLine.Y2 = zeroY;
            }
        }
        
        private void DrawHorizontalLineAndLabel(double db, double w, double h)
        {
            double y = DbToY(db, h);
            EqGridCanvas.Children.Add(new Line { X1 = ScreenPaddingX, Y1 = y, X2 = w - ScreenPaddingX, Y2 = y, Stroke = Brushes.Gray, Opacity = 0.15 });
            
            var tb = new TextBlock
            {
                Text = (db > 0 ? "+" : "") + db,
                FontSize = 10, Foreground = Brushes.Gray,
                HorizontalAlignment = HorizontalAlignment.Right,
                TextAlignment = TextAlignment.Right,
                Width = 25
            };
            Canvas.SetLeft(tb, 0);
            Canvas.SetTop(tb, y - 6);
            EqGridCanvas.Children.Add(tb);
        }

        private void UpdateNoiseGateVisual()
        {
            if (NoiseGateLine == null) return;
            
            double w = EqCanvas.ActualWidth > 0 ? EqCanvas.ActualWidth : 450;
            double h = EqCanvas.ActualHeight > 0 ? EqCanvas.ActualHeight : 240;
            
            double db = NoiseGateSlider.Value;
            double y = DbToY(db, h);
            
            NoiseGateLine.X1 = ScreenPaddingX;
            NoiseGateLine.X2 = w - ScreenPaddingX;
            NoiseGateLine.Y1 = y;
            NoiseGateLine.Y2 = y;
        }

        private void EqCanvas_MouseLeftButtonDown(object sender, MouseButtonEventArgs e)
        {
            var pos = e.GetPosition(EqCanvas);
            
            if (FlatModeCheck.IsChecked == true)
            {
                double targetGain = YToDb(pos.Y, EqCanvas.ActualHeight);
                foreach(var p in _uiPoints) p.Gain = targetGain;
                
                // Start dragging logic (hijack first point or dummy)
                // We need DraggingPoint not null to trigger MouseMove
                _draggingPoint = _uiPoints.FirstOrDefault(); 
                
                EqCanvas.CaptureMouse();
                UpdatePointVisuals();
                UpdateEqCurve(); // Added: Update curve immediately on click
                UpdateAudioEq();
                SaveSettings(); 
                return;
            }

            // Hit test
            foreach (var p in _uiPoints)
            {
                double px = CoordinateX(p.Freq, EqCanvas.ActualWidth); 
                double py = CoordinateY(p.Gain, EqCanvas.ActualHeight);
                
                if (Math.Abs(px - pos.X) < 12 && Math.Abs(py - pos.Y) < 12)
                {
                    // Flat Mode Restriction: Can only drag 20Hz point
                    if (FlatModeCheck.IsChecked == true && Math.Abs(p.Freq - 20) > 1.0) return;

                    _draggingPoint = p;
                    EqCanvas.CaptureMouse();
                    return;
                }
            }

            // Flat Mode Restriction: No adding points
            if (FlatModeCheck.IsChecked == true) return;

            // Check padding bounds for new points
            if (pos.X < ScreenPaddingX || pos.X > EqCanvas.ActualWidth - ScreenPaddingX) return;
            if (pos.Y < ScreenPaddingY || pos.Y > EqCanvas.ActualHeight - ScreenPaddingY) return;

            // Add new Peaking point
            if (_uiPoints.Count < 16)
            {
                double freq = XToFreq(pos.X, EqCanvas.ActualWidth);
                double gain = YToDb(pos.Y, EqCanvas.ActualHeight);
                var newPoint = AddPointToCanvas(freq, gain, 1.0f, BandType.Peaking);
                _draggingPoint = newPoint;
                EqCanvas.CaptureMouse();
                UpdateAudioEq();
                RefreshAllVisuals();
            }
        }

        private void EqCanvas_MouseMove(object sender, MouseEventArgs e)
        {
            if (_draggingPoint != null)
            {
                var pos = e.GetPosition(EqCanvas);
                double w = EqCanvas.ActualWidth;
                double h = EqCanvas.ActualHeight;
                
                // Clamp to padding area
                pos.X = Math.Max(ScreenPaddingX, Math.Min(w - ScreenPaddingX, pos.X));
                pos.Y = Math.Max(ScreenPaddingY, Math.Min(h - ScreenPaddingY, pos.Y));

                bool isFlatMode = FlatModeCheck.IsChecked == true;

                if (isFlatMode)
                {
                    // Flat Mode: Update ALL points to same gain
                    double targetGain = YToDb(pos.Y, h);
                    foreach(var p in _uiPoints) p.Gain = targetGain;
                }
                else
                {
                    // Point Mode
                    // Lock Freq (X-axis) for Edge Points (Min/Max Freq aka Shelves)
                    bool isEdge = (Math.Abs(_draggingPoint.Freq - _minFreq) < 0.1) || (Math.Abs(_draggingPoint.Freq - _maxFreq) < 0.1) || _draggingPoint.Type != BandType.Peaking;
                    
                    if (!isEdge)
                    {
                        _draggingPoint.Freq = XToFreq(pos.X, w);
                    }
                    _draggingPoint.Gain = YToDb(pos.Y, h);
                }

                UpdatePointVisuals();
                UpdateEqCurve();
                UpdateAudioEq(); // Added: Real-time audio update during drag
            }
        }

        private void EqCanvas_MouseLeftButtonUp(object sender, MouseButtonEventArgs e)
        {
            if (_draggingPoint != null)
            {
                _draggingPoint = null;
                EqCanvas.ReleaseMouseCapture();
                UpdateAudioEq();
                SaveSettings();
            }
        }
        
        private void EqCanvas_MouseLeave(object sender, MouseEventArgs e)
        {
             if (_draggingPoint != null) { _draggingPoint = null; EqCanvas.ReleaseMouseCapture(); UpdateAudioEq(); SaveSettings(); }
        }

        private void EqCanvas_MouseWheel(object sender, MouseWheelEventArgs e)
        {
            var pos = e.GetPosition(EqCanvas);
            UiEqPoint? target = null;
            
            foreach (var p in _uiPoints)
            {
                double px = CoordinateX(p.Freq, EqCanvas.ActualWidth);
                double py = CoordinateY(p.Gain, EqCanvas.ActualHeight);
                if (Math.Abs(px - pos.X) < 15 && Math.Abs(py - pos.Y) < 15) { target = p; break; }
            }

            if (target != null)
            {
                /* 
                   User Step 1978: Disable Q adjustment via GUI.
                */
                // Force 0.1 just in case?
                target.Q = 0.1f;
                
                UpdateAudioEq();
                RefreshAllVisuals();
                SaveSettings();
            }
        }

        private void EqCanvas_MouseRightButtonDown(object sender, MouseButtonEventArgs e)
        {
            var pos = e.GetPosition(EqCanvas);
            UiEqPoint? target = null;
            foreach (var p in _uiPoints)
            {
                double px = CoordinateX(p.Freq, EqCanvas.ActualWidth);
                double py = CoordinateY(p.Gain, EqCanvas.ActualHeight);
                if (Math.Abs(px - pos.X) < 15 && Math.Abs(py - pos.Y) < 15) { target = p; break; }
            }

            if (target != null && target.Type == BandType.Peaking)
            {
                _uiPoints.Remove(target);
                EqCanvas.Children.Remove(target.Visual);
                UpdateAudioEq();
                RefreshAllVisuals();
                SaveSettings();
            }
        }

        private UiEqPoint AddPointToCanvas(double freq, double gain, float q, BandType type)
        {
            float finalQ = 0.1f; // Force Q to 0.1
            
            var ellipse = new Ellipse
            {
                Width = 14, Height = 14, // Slightly larger
                Fill = (type == BandType.Peaking && freq > _minFreq && freq < _maxFreq) ? (Brush)FindResource("AccentBrush") : Brushes.OrangeRed,
                Stroke = Brushes.White, StrokeThickness = 2,
                Tag = type // Marker
            };
            
            // Adjust Z-Index so points are above lines
            Panel.SetZIndex(ellipse, 10);
            
            var point = new UiEqPoint { Freq = freq, Gain = gain, Q = finalQ, Type = type, Visual = ellipse };
            _uiPoints.Add(point);
            EqCanvas.Children.Add(ellipse);
            
            return point;
        }

        private void UpdatePointVisuals()
        {
            if (EqCanvas.ActualWidth == 0) return;
            double w = EqCanvas.ActualWidth;
            double h = EqCanvas.ActualHeight;

            foreach (var p in _uiPoints)
            {
                double x = CoordinateX(p.Freq, w);
                double y = CoordinateY(p.Gain, h);
                
                // Hide points outside visible range (except maybe shelves on edge?)
                bool visible = p.Freq >= _minFreq && p.Freq <= _maxFreq;

                // Flat Mode: Show everything (Left/Right dots)
                // User requirement: "Left and Right circles should be Red".
                if (FlatModeCheck.IsChecked == true)
                {
                    // In Flat mode, essentially 2 points (min/max).
                    // If others exist, hide them? 
                    // But Logic says "Update ALL points".
                    // If user has manual points, Flat mode flattens them all.
                    // For clarity, maybe only show Min/Max in Flat Mode?
                    // And flatten logic in MouseMove updates all.
                    
                    // Actually, Flat Mode usually implies just a Gain slider.
                    // If we hide intermediate points, it's cleaner.
                    
                    if (p.Freq > _minFreq + 1 && p.Freq < _maxFreq - 1) visible = false;
                }
                // Exception for shelves: clamp them visualy to edges if active?
                // No, just hide them if they are strictly out of range, to avoid confusion.
                // But users might lose them. Let's clamp shelves visually if enabled.
                if (p.Type != BandType.Peaking)
                {
                     // If Shelf is 20Hz and Min is 100Hz -> x will be negative.
                     // Clamp to padding?
                     if (x < ScreenPaddingX) x = ScreenPaddingX;
                     if (x > w - ScreenPaddingX) x = w - ScreenPaddingX;
                     visible = true; // Always show shelves at edge
                }

                p.Visual.Visibility = visible ? Visibility.Visible : Visibility.Collapsed;
                if (visible)
                {
                    Canvas.SetLeft(p.Visual, x - 7);
                    Canvas.SetTop(p.Visual, y - 7);
                    // Q removed from display
                    p.Visual.ToolTip = $"{p.Type}\nFreq: {p.Freq:F0}Hz\nGain: {p.Gain:F1}dB";
                }
            }
        }

        private void UpdateEqCurve()
        {
            if (EqCanvas.ActualWidth == 0) return;
            
            double w = EqCanvas.ActualWidth;
            double h = EqCanvas.ActualHeight;
            PointCollection points = new PointCollection();
            
            // EQ is always enabled, no bypass check here.
            // If EqBypassCheck was a UI element for visual bypass, it's removed.
            // The actual audio engine bypass is handled by UpdateAudioEq()
            
            // Linear Graph: Just connect the sorted points
            var sorted = _uiPoints.OrderBy(p => p.Freq).ToList();
            if (sorted.Count == 0) return;

            // Ensure start at min (20Hz)
            if (sorted[0].Freq > 20)
            {
                // extrapolation or flat from first point?
                // Logic: 20Hz point always exists.
            }
            
            foreach (var p in sorted)
            {
                double x = CoordinateX(p.Freq, w);
                double y = CoordinateY(p.Gain, h);
                points.Add(new Point(x, y));
            }
            
            if (EqResponsePolyline != null) EqResponsePolyline.Points = points;
        }

        private double GetInterpolatedDb(double freq, List<UiEqPoint> sortedPoints)
        {
            if (sortedPoints.Count == 0) return 0;
            if (freq <= sortedPoints[0].Freq) return sortedPoints[0].Gain;
            if (freq >= sortedPoints.Last().Freq) return sortedPoints.Last().Gain;

            // Find segment
            for (int i = 0; i < sortedPoints.Count - 1; i++)
            {
                var p1 = sortedPoints[i];
                var p2 = sortedPoints[i+1];
                if (freq >= p1.Freq && freq <= p2.Freq)
                {
                    // Linear interpolation on Log Freq scale
                    double logF = Math.Log10(freq);
                    double logF1 = Math.Log10(p1.Freq);
                    double logF2 = Math.Log10(p2.Freq);
                    double t = (logF - logF1) / (logF2 - logF1);
                    return p1.Gain + (t * (p2.Gain - p1.Gain));
                }
            }
            return 0;
        }

        private void OnFftDataReceived(object? sender, float[] fftData)
        {
            var now = DateTime.Now;
            if ((now - _lastSpectrumUpdate).TotalMilliseconds < 33) return;
            _lastSpectrumUpdate = now;
            _currentSpectrum = fftData;

            Dispatcher.BeginInvoke(System.Windows.Threading.DispatcherPriority.Background, () =>
            {
                if (SpectrumPolyline == null || EqCanvas.ActualWidth == 0 || _currentSpectrum == null) return;
                
                double w = EqCanvas.ActualWidth;
                double h = EqCanvas.ActualHeight;
                PointCollection points = new PointCollection();
                
                double bottomY = h - ScreenPaddingY;
                points.Add(new Point(ScreenPaddingX, bottomY)); 

                for (double x = ScreenPaddingX; x <= w - ScreenPaddingX; x += 4)
                {
                    double f = XToFreq(x, w);
                    // 44100Hz / 2048 = 21.5Hz bin
                    int binIndex = (int)(f / 21.53);
                    
                    if (binIndex >= 0 && binIndex < _currentSpectrum.Length)
                    {
                        double val = _currentSpectrum[binIndex]; 
                        
                        // Map 0..1 to Display Height
                        // 0 means -60dB, 1 means 0dB (Relative to Max)
                        // But wait! AudioEngine sends normalized dB values.
                        // We want to visualize roughly where the sound is.
                        // Let's use full height available inside padding.
                        double graphHeight = h - (ScreenPaddingY * 2);
                        
                        // Apply rudimentary smoothing or gain boost for visibility?
                        // If it's too quiet, boost it visually?
                        // val is 0.0-1.0. 
                        
                        double y = bottomY - (val * graphHeight);
                        points.Add(new Point(x, y));
                    }
                }
                
                points.Add(new Point(w - ScreenPaddingX, bottomY));
                SpectrumPolyline.Points = points;
            });
        }
        
        // =========================================================================
        // Coordinate Helpers with Padding
        // =========================================================================
        
        // =========================================================================
        // Coordinate Helpers with Padding
        // =========================================================================
        
        // Freq <-> X
        private double FreqToX(double freq, double canvasWidth)
        {
            double safeW = canvasWidth - (ScreenPaddingX * 2);
            double minLog = Math.Log10(_minFreq);
            double maxLog = Math.Log10(_maxFreq);
            double fLog = Math.Log10(Math.Max(_minFreq, Math.Min(_maxFreq, freq)));
            
            double ratio = (fLog - minLog) / (maxLog - minLog);
            return ScreenPaddingX + (ratio * safeW);
        }

        private double XToFreq(double x, double canvasWidth)
        {
            double safeW = canvasWidth - (ScreenPaddingX * 2);
            double localX = x - ScreenPaddingX;
            double ratio = localX / safeW;
            
            double minLog = Math.Log10(_minFreq);
            double maxLog = Math.Log10(_maxFreq); // _minFreq to _maxFreq range
            double fLog = minLog + (ratio * (maxLog - minLog));
            return Math.Pow(10, fLog);
        }

        // dB <-> Y
        private double DbToY(double db, double canvasHeight)
        {
            double safeH = canvasHeight - (ScreenPaddingY * 2);
            double range = _currentMaxDb * 2;
            double ratio = (_currentMaxDb - db) / range;
            return ScreenPaddingY + (ratio * safeH);
        }

        private double YToDb(double y, double canvasHeight)
        {
            double safeH = canvasHeight - (ScreenPaddingY * 2);
            double localY = y - ScreenPaddingY;
            double ratio = localY / safeH;
            double range = _currentMaxDb * 2;
            return _currentMaxDb - (ratio * range);
        }
        
        // Public Accessors for update loop
        private double CoordinateX(double freq, double w) => FreqToX(freq, w);
        private double CoordinateY(double gain, double h) => DbToY(gain, h);

        protected override void OnRenderSizeChanged(SizeChangedInfo sizeInfo)
        {
            base.OnRenderSizeChanged(sizeInfo);
            if (!_isInitializing && sizeInfo.WidthChanged) RefreshAllVisuals();
        }

        // Standard helpers
        private void UpdateAudioEq() 
        {
            if (_audioEngine == null) return;

            // Parametric EQ Conversion (V5.1+)
            // Use actual UI points as filters instead of resampling 31 bands
            var sorted = _uiPoints.OrderBy(p => p.Freq).ToList();

            var bands = new List<EqBand>();
            double masterDb = 0;
            
            // Check Main Toggle
            bool isBoostEnabled = BoostToggle?.IsChecked ?? true;

            if (!isBoostEnabled)
            {
                // BYPASS MODE: 0dB Gain, Flat EQ
                masterDb = 0; 
                // Don't add any bands -> Flat
                _audioEngine.UpdateParametricEq(0.0f, new List<EqBand>());
                return;
            }
            
            // Volume Handling logic
            if (FlatModeCheck.IsChecked == true && sorted.Count > 0) 
            {
                masterDb = sorted[0].Gain; 
                // Filters are 0dB (Flat) relative to Master
                // Since _uiPoints are all updated to same gain, (Gain - Master) is 0.
            }
            else
            {
                masterDb = 0; // Point Mode: Filters handle gains
            }

            foreach (var p in sorted)
            {
                float relGain = (float)(p.Gain - masterDb);
                
                bands.Add(new EqBand 
                {
                    Frequency = (float)p.Freq,
                    Gain = relGain,
                    Q = (float)p.Q,
                    Type = p.Type
                });
            }
            
            // Send to AudioEngine
            _audioEngine.UpdateParametricEq((float)masterDb, bands);
        }
        private void AddPoint_Click(object sender, RoutedEventArgs e)
        {
            if (FlatModeCheck.IsChecked == true)
            {
                MessageBox.Show("フラットモード中はポイントを追加できません。", "制限", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            if (_uiPoints.Count >= 16)
            {
                MessageBox.Show("ポイント数の上限(16)に達しました。", "制限", MessageBoxButton.OK, MessageBoxImage.Information);
                return;
            }

            // Add center point freq (1kHz) at 0dB (or center of view?)
            // Or better: midpoint between min/max freq
            // Log center: sqrt(20 * 20000) = 632 Hz approx.
            double centerFreq = Math.Sqrt(_minFreq * _maxFreq);
            AddPointToCanvas(centerFreq, 0, 0.1f, BandType.Peaking);
            
            UpdateAudioEq();
            RefreshAllVisuals();
            SaveSettings();
        }

        private void UnlockLimit_Changed(object sender, RoutedEventArgs e)
        {
            if (_isInitializing) return;
            _currentMaxDb = UnlockLimitCheck.IsChecked == true ? 100 : 50;
            NoiseGateSlider.Minimum = -_currentMaxDb; 
            NoiseGateSlider.Maximum = _currentMaxDb; // Update Max
            
            RefreshAllVisuals();
            SaveSettings();
        }

        private void BoostToggle_Changed(object sender, RoutedEventArgs e)
        {
             if (_isInitializing) return;
             UpdateAudioEq();
             SaveSettings();
        }

        // Preset Handlers
        private void PresetCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (_isInitializing || _isLoadingPreset) return;
            if (PresetCombo.SelectedItem is Preset p)
            {
                ApplyPreset(p);
            }
        }

        private void ApplyPreset(Preset p)
        {
            _isLoadingPreset = true;
            try
            {
                // Clear existing
                // Clear old visuals
                var toRemove = EqCanvas.Children.OfType<Ellipse>().ToList();
                foreach (var el in toRemove) EqCanvas.Children.Remove(el);
                _uiPoints.Clear();
                
                // Apply Params
                FlatModeCheck.IsChecked = p.FlatMode;
                UnlockLimitCheck.IsChecked = p.UnlockLimit;
                NoiseGateSlider.Value = p.NoiseGateThreshold;
                MinFreqInput.Text = p.MinFreq.ToString();
                MaxFreqInput.Text = p.MaxFreq.ToString();
                
                _minFreq = p.MinFreq > 0 ? p.MinFreq : 20;
                _maxFreq = p.MaxFreq > 0 ? p.MaxFreq : 20000;
                
                // Load Bands
                foreach (var b in p.Bands)
                {
                    AddPointToCanvas(b.Frequency, b.Gain, 0.1f, b.Type);
                }
                
                // If bands empty (e.g. flat default), ensure at least one if logic requires it? 
                // Or FlatMode handles it (AddPointToCanvas logic).
                
                UpdateAudioEq();
                RefreshAllVisuals();
                SaveSettings();
            }
            finally
            {
                _isLoadingPreset = false;
            }
        }

        private void SavePreset_Click(object sender, RoutedEventArgs e)
        {
            string name = PresetCombo.Text.Trim();
            if (string.IsNullOrEmpty(name))
            {
                MessageBox.Show("プリセット名を入力してください。", "エラー", MessageBoxButton.OK, MessageBoxImage.Warning);
                return;
            }

            var existing = _presets.FirstOrDefault(p => p.Name.Equals(name, StringComparison.OrdinalIgnoreCase));
            if (existing != null)
            {
                var res = MessageBox.Show($"プリセット '{name}' は既に存在します。上書きしますか？", "確認", MessageBoxButton.YesNo, MessageBoxImage.Question);
                if (res != MessageBoxResult.Yes) return;
                
                // Update existing
                existing.FlatMode = FlatModeCheck.IsChecked ?? false;
                existing.Bands = _uiPoints.Select(p => new EqBand { Frequency = (float)p.Freq, Gain = (float)p.Gain, Q = p.Q, Type = p.Type }).ToList();
                existing.UnlockLimit = UnlockLimitCheck.IsChecked ?? false;
                existing.NoiseGateThreshold = (float)NoiseGateSlider.Value;
                if (double.TryParse(MinFreqInput.Text, out double min)) existing.MinFreq = min;
                if (double.TryParse(MaxFreqInput.Text, out double max)) existing.MaxFreq = max;
            }
            else
            {
                // New
                var p = new Preset
                {
                    Name = name,
                    FlatMode = FlatModeCheck.IsChecked ?? false,
                    Bands = _uiPoints.Select(pt => new EqBand { Frequency = (float)pt.Freq, Gain = (float)pt.Gain, Q = pt.Q, Type = pt.Type }).ToList(),
                    UnlockLimit = UnlockLimitCheck.IsChecked ?? false,
                    NoiseGateThreshold = (float)NoiseGateSlider.Value
                };
                if (double.TryParse(MinFreqInput.Text, out double min)) p.MinFreq = min;
                if (double.TryParse(MaxFreqInput.Text, out double max)) p.MaxFreq = max;
                
                _presets.Add(p);
            }

            _configManager.SavePreset(existing ?? _presets.Last());
            UpdatePresetCombo();
            // Select the saved one
            PresetCombo.SelectedItem = _presets.FirstOrDefault(x => x.Name == name);
        }

        private void DeletePreset_Click(object sender, RoutedEventArgs e)
        {
            if (PresetCombo.SelectedItem is Preset p)
            {
                 var res = MessageBox.Show($"プリセット '{p.Name}' を削除しますか？", "確認", MessageBoxButton.YesNo, MessageBoxImage.Question);
                 if (res == MessageBoxResult.Yes)
                 {
                     _presets.Remove(p);
                     _configManager.DeletePreset(p.Name);
                     UpdatePresetCombo();
                     PresetCombo.Text = "";
                 }
            }
        }

        private void OpenPresetFolder_Click(object sender, RoutedEventArgs e)
        {
             string path = System.IO.Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "Presets");
             if (!System.IO.Directory.Exists(path)) System.IO.Directory.CreateDirectory(path);
             Process.Start("explorer.exe", path);
        }

        private void FlatMode_Changed(object sender, RoutedEventArgs e)
        {
            if (_isInitializing) return;
            
            bool isFlat = FlatModeCheck.IsChecked == true;
            
            if (isFlat)
            {
                // Switch to Flat: Keep 20Hz gain, clear others
                double currentMasterGain = 0;
                var lowPt = _uiPoints.FirstOrDefault(p => Math.Abs(p.Freq - 20) < 1.0);
                if (lowPt != null) currentMasterGain = lowPt.Gain;
                
                // Clear visuals
                 var toRemove = EqCanvas.Children.OfType<Ellipse>().ToList();
                foreach (var el in toRemove) EqCanvas.Children.Remove(el);
                _uiPoints.Clear();

                // Add Flat Points (20Hz and 20kHz)
                AddPointToCanvas(20, currentMasterGain, 0.7f, BandType.LowShelf);
                var highPt = AddPointToCanvas(20000, currentMasterGain, 0.7f, BandType.HighShelf);
                
                // Hide High Point?
                // User said "Flat mode: left red point ONLY".
                // So hide the 20kHz point visually but keep it for logic.
                highPt.Visual.Visibility = Visibility.Collapsed; // Will be handled in UpdatePointVisuals if I add logic there, but manual set works if UpdatePointVisuals doesn't override it immediately. 
                // Wait, UpdatePointVisuals refreshes visibility based on range. 
                // I need UpdatePointVisuals to know about FlatMode or just ignore it? 
                // Let's modify UpdatePointVisuals or just set opacity to 0 in AddPoint?
                // I'll update UpdatePointVisuals logic or just rely on the fact that I won't call UpdatePointVisuals on it?
                // UpdatePointVisuals is called on Move.
                // Better to handle "Hidden 20kHz" properly.
                // Let's add a Tag or special check in UpdatePointVisuals?
                // Or just: In Flat Mode, hide anything > 20Hz.
            }
            // If Unchecked (Point Mode), we just stay where we are (Flat). User adds points manually.

            RefreshAllVisuals();
            UpdateAudioEq();
            SaveSettings();
        }

        // EqBypass_Changed Removed

        private void SaveSettings()
        {
            if (_isInitializing || _configManager == null) return;

            // Always enabled
            // Freq range not saved in Config yet, but requested in layout changes? 
            // User query: "Default min20 max20000".
            // Since Preset has it, Config should probably have it too for consistency?
            // For now, Stick to existing Config Write, but pass true for enabled.
            
            var input = InputDeviceCombo.SelectedItem as MMDevice; 
            var output = OutputDeviceCombo.SelectedItem as MMDevice; 

            _configManager.WriteConfig(1.0f, BoostToggle?.IsChecked ?? true, input?.ID ?? "", output?.ID ?? "", 
                                     UnlockLimitCheck.IsChecked == true,
                                     _uiPoints.Select(p => new EqBand { Frequency = (float)p.Freq, Gain = (float)p.Gain, Q = p.Q, Type = p.Type }).ToList(),
                                     (float)NoiseGateSlider.Value,
                                     0.0f, // Fixed Limiter Threshold
                                     true, // Fixed Limiter Enabled
                                     FlatModeCheck.IsChecked == true,
                                     PresetCombo.Text,
                                     Localization.LocalizationManager.Instance.CurrentLanguage,
                                     true); // Always enabled
            
            // Clear Preset Name on user edit logic REMOVED (User request V6.1)
        }

        private void Language_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (_isInitializing) return;
            if (LanguageCombo.SelectedItem is ComboBoxItem item && item.Tag is string code)
            {
                Localization.LocalizationManager.Instance.LoadLanguage(code);
                SaveSettings();
            }
        }

        private void ReloadPreset_Click(object sender, RoutedEventArgs e)
        {
            if (PresetCombo.SelectedItem is Preset p)
            {
                ApplyPreset(p);
            }
        }

        private void Preset_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (_isInitializing) return;
            if (PresetCombo.SelectedItem is Preset p)
            {
                ApplyPreset(p);
            }
        }

        private void NoiseGateSlider_ValueChanged(object sender, RoutedPropertyChangedEventArgs<double> e)
        {
            if (_isInitializing) return;
            if (_audioEngine == null) return;
            if (NoiseGateValueText != null) NoiseGateValueText.Text = $"{(e.NewValue > 0 ? "+" : "")}{e.NewValue:F0}dB";
            _audioEngine?.SetNoiseGateThreshold((float)e.NewValue);
            UpdateNoiseGateVisual();
            SaveSettings();
        }
        



        

        private void Device_SelectionChanged(object sender, SelectionChangedEventArgs e) 
        { 
            if(_isInitializing) return; 
            UpdateDeviceValidation();
            
            // Always restart audio when device changes if both devices are selected
            var input = InputDeviceCombo.SelectedItem as MMDevice;
            var output = OutputDeviceCombo.SelectedItem as MMDevice;
            if (input != null && output != null)
            {
                StopAudio();
                StartAudio();
            }
            SaveSettings(); 
        }
        
        private void UpdateDeviceValidation()
        {
            bool hasInput = InputDeviceCombo.SelectedItem != null;
            bool hasOutput = OutputDeviceCombo.SelectedItem != null;
            bool isValid = hasInput && hasOutput;
            
            // Red border for missing devices (using Border wrapper)
            InputDeviceBorder.BorderBrush = hasInput ? new SolidColorBrush((Color)ColorConverter.ConvertFromString("#555555")) : new SolidColorBrush(Colors.Red);
            InputDeviceBorder.BorderThickness = hasInput ? new Thickness(1) : new Thickness(2);
            
            OutputDeviceBorder.BorderBrush = hasOutput ? new SolidColorBrush((Color)ColorConverter.ConvertFromString("#555555")) : new SolidColorBrush(Colors.Red);
            OutputDeviceBorder.BorderThickness = hasOutput ? new Thickness(1) : new Thickness(2);
            
            // Toggle state based on device selection
            BoostToggle.IsEnabled = isValid;
            
            if (!isValid)
            {
                // Devices not selected: gray indicator, disabled state
                BoostToggle.IsChecked = false;
            }
        }
        
        private void RefreshDevices_Click(object sender, RoutedEventArgs e) { LoadDevices(); UpdateDeviceValidation(); }
        private void StartAudio() { 
            var i=InputDeviceCombo.SelectedItem as MMDevice; var o=OutputDeviceCombo.SelectedItem as MMDevice; 
            if(i==null||o==null){ UpdateDeviceValidation(); return;} 
            if(i.ID==o.ID){MessageBox.Show(Localization.LocalizationManager.Instance.GetString("Msg_Loop"));return;} 
            
            UpdateAudioEq(); // Calculates proper gains
            // Actually UpdateAudioEq calls UpdateGraphicEq which sets _gain on engine. 
            // But Start() needs initial values.
            // Let's modify Start() to NOT take bands, but we need intial gain.
            // We can call Start then UpdateAudioEq. 
            // Start(i, o, 1.0f, noiseGate); -> Gain defaults to 1.0 then updated.
            
             // Default gain 1.0 (0dB) to avoid blast if toggle is off but engine applies old state?
             // Actually, pass 0dB initial. UpdateAudioEq will fix it immediately.
            _audioEngine.Start(i,o, 1.0f, (float)NoiseGateSlider.Value);
            UpdateAudioEq(); // Apply EQ/Volume immediately
        }
        private void StopAudio() { _audioEngine.Stop(); if(LevelMeter!=null) LevelMeter.Width=0; }


        private void OnPeakLevelReceived(object? s, float v) { Dispatcher.InvokeAsync(()=>{if(LevelMeter!=null&&ActualWidth>0){LevelMeter.MaxWidth=Math.Max(0,ActualWidth-80);LevelMeter.Width=LevelMeter.MaxWidth*v;}}); }
        private void OnAudioError(object? s, string m) { Dispatcher.InvokeAsync(()=>{StopAudio();MessageBox.Show(m);}); }
        private void StartupCheckbox_Changed(object s, RoutedEventArgs e) { if(!_isInitializing) SetStartup(StartupCheckbox.IsChecked??false); }
        private bool IsStartupEnabled() { try{ using var k=Registry.CurrentUser.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",false); return k?.GetValue("EasyMICBooster")!=null;}catch{return false;} }
        private void SetStartup(bool e) { try{ using var k=Registry.CurrentUser.OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",true); if(k!=null) if(e) k.SetValue("EasyMICBooster",$"\"{System.Diagnostics.Process.GetCurrentProcess().MainModule?.FileName}\""); else k.DeleteValue("EasyMICBooster",false); }catch{} }
        protected override void OnStateChanged(EventArgs e) { base.OnStateChanged(e); if(WindowState==WindowState.Minimized)Hide(); }
        protected override void OnClosing(System.ComponentModel.CancelEventArgs e) 
        { 
            if (!App.IsExitingFromTray)
            {
                if (MessageBox.Show(Localization.LocalizationManager.Instance.GetString("Msg_ExitConfirm"), 
                                    Localization.LocalizationManager.Instance.GetString("Msg_Confirm"), 
                                    MessageBoxButton.YesNo, MessageBoxImage.Question) != MessageBoxResult.Yes)
                {
                    e.Cancel = true;
                    return;
                }
            }
            _audioEngine?.Dispose(); 
            base.OnClosing(e); 
        }

        protected override void OnContentRendered(EventArgs e)
        {
            base.OnContentRendered(e);
            CheckForUpdatesIfNeeded();
        }

        private string? _latestVersion;

        private async void CheckForUpdatesIfNeeded()
        {
            if (_hasCheckedUpdate) return;
            _hasCheckedUpdate = true;
            
            // Show "確認中" state
            UpdateStatusText.Text = Localization.LocalizationManager.Instance.GetString("Msg_UpdateChecking");
            UpdateStatusText.Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#888888"));
            UpdateStatusText.TextDecorations = null;
            
            var (isNewer, latest, error) = await VersionManager.CheckForUpdateAsync();
            
            if (error != null)
            {
                // Show error state
                UpdateStatusText.Text = Localization.LocalizationManager.Instance.GetString("Msg_UpdateError");
                UpdateStatusText.Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#FF6B6B"));
                UpdateStatusText.TextDecorations = null;
                return;
            }
            
            if (isNewer)
            {
                _latestVersion = latest;
                UpdateStatusText.Text = string.Format(Localization.LocalizationManager.Instance.GetString("Msg_UpdateAvailable"), latest);
                UpdateStatusText.Foreground = new SolidColorBrush((Color)ColorConverter.ConvertFromString("#4CAF50"));
                UpdateStatusText.TextDecorations = TextDecorations.Underline;
            }
            else
            {
                // Latest version, clear text
                UpdateStatusText.Text = "";
            }
        }

        private void UpdateStatusText_Click(object sender, MouseButtonEventArgs e)
        {
            if (!string.IsNullOrEmpty(_latestVersion))
            {
                Process.Start(new ProcessStartInfo { FileName = "https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases", UseShellExecute = true });
            }
        }
    }
}
