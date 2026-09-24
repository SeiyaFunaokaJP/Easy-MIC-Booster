using System;
using System.Collections.Generic;
using NAudio.CoreAudioApi;
using NAudio.CoreAudioApi.Interfaces;

namespace EasyMICBooster
{
    // ComboBox item wrapping either a concrete endpoint or the "follow Windows default" pseudo-device.
    public class DeviceListItem
    {
        public const string DefaultDeviceId = "@DEFAULT_DEVICE";

        public MMDevice? Device { get; init; }
        public string Id { get; init; } = "";
        public string DisplayName { get; init; } = "";
        public bool IsDefault => Id == DefaultDeviceId;

        public static List<DeviceListItem> GetInputList() => BuildList(DataFlow.Capture);
        public static List<DeviceListItem> GetOutputList() => BuildList(DataFlow.Render);

        // Resolves the pseudo-device to whatever Windows currently considers the default endpoint.
        public static MMDevice? ResolveDefault(DataFlow flow)
        {
            try
            {
                using var enumerator = new MMDeviceEnumerator();
                if (!enumerator.HasDefaultAudioEndpoint(flow, Role.Console)) return null;
                return enumerator.GetDefaultAudioEndpoint(flow, Role.Console);
            }
            catch
            {
                return null;
            }
        }

        private static List<DeviceListItem> BuildList(DataFlow flow)
        {
            var list = new List<DeviceListItem>();
            var loc = Localization.LocalizationManager.Instance;

            var defaultDevice = ResolveDefault(flow);
            string defaultName = defaultDevice != null
                ? string.Format(loc.GetString("Device_Default"), defaultDevice.FriendlyName)
                : loc.GetString("Device_Default_None");
            list.Add(new DeviceListItem { Id = DefaultDeviceId, DisplayName = defaultName });

            using var enumerator = new MMDeviceEnumerator();
            foreach (var d in enumerator.EnumerateAudioEndPoints(flow, DeviceState.Active))
            {
                list.Add(new DeviceListItem { Id = d.ID, DisplayName = d.FriendlyName, Device = d });
            }
            return list;
        }

        public override string ToString() => DisplayName;
    }

    // Forwards WASAPI endpoint notifications (device arrival/removal/default change) as a single callback.
    // Callbacks arrive on an MTA worker thread; the receiver must marshal to the UI thread.
    public class DeviceNotificationClient : IMMNotificationClient
    {
        private readonly Action _changed;

        public DeviceNotificationClient(Action changed) => _changed = changed;

        public void OnDeviceStateChanged(string deviceId, DeviceState newState) => _changed();
        public void OnDeviceAdded(string pwstrDeviceId) => _changed();
        public void OnDeviceRemoved(string deviceId) => _changed();

        public void OnDefaultDeviceChanged(DataFlow flow, Role role, string defaultDeviceId)
        {
            // Fires once per role; Console is the role we resolve against.
            if (role == Role.Console) _changed();
        }

        public void OnPropertyValueChanged(string pwstrDeviceId, PropertyKey key) { }
    }
}
