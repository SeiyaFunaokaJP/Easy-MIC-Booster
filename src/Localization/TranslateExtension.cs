using System;
using System.Windows.Data;
using System.Windows.Markup;

namespace EasyMICBooster.Localization
{
    [MarkupExtensionReturnType(typeof(string))]
    public class Tr : MarkupExtension
    {
        public string Key { get; set; }

        public Tr(string key)
        {
            this.Key = key;
        }

        public override object ProvideValue(IServiceProvider serviceProvider)
        {
            var binding = new Binding
            {
                Source = LocalizationManager.Instance,
                Path = new System.Windows.PropertyPath($"[{Key}]"),
                Mode = BindingMode.OneWay
            };
            return binding.ProvideValue(serviceProvider);
        }
    }
}
