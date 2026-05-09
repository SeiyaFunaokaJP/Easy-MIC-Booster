using System;
using System.Runtime.InteropServices;

namespace EasyMICBooster
{
    internal static class RNNoiseInterop
    {
        public const int FrameSize = 480;
        public const int SampleRate = 48000;
        private const string Dll = "rnnoise";

        [DllImport(Dll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rnnoise_create")]
        public static extern IntPtr Create(IntPtr model);

        [DllImport(Dll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rnnoise_destroy")]
        public static extern void Destroy(IntPtr state);

        [DllImport(Dll, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rnnoise_process_frame")]
        public static extern float ProcessFrame(IntPtr state, float[] output, float[] input);

        private static bool? _availableCache;

        public static bool IsAvailable
        {
            get
            {
                if (_availableCache.HasValue) return _availableCache.Value;
                _availableCache = Probe();
                return _availableCache.Value;
            }
        }

        private static bool Probe()
        {
            try
            {
                var st = Create(IntPtr.Zero);
                if (st == IntPtr.Zero) return false;
                Destroy(st);
                return true;
            }
            catch (DllNotFoundException) { return false; }
            catch (BadImageFormatException) { return false; }
            catch (EntryPointNotFoundException) { return false; }
            catch { return false; }
        }
    }
}
