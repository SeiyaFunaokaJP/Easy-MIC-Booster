using System;
using System.Windows;
using Hardcodet.Wpf.TaskbarNotification;

namespace EasyMICBooster
{
    public partial class App : Application
    {
        private TaskbarIcon? _trayIcon;

        private System.Threading.Mutex? _mutex;
        public static bool IsExitingFromTray = false;

        protected override void OnStartup(StartupEventArgs e)
        {
            const string mutexName = "Global\\EasyMICBooster_UniqueInstance_Mutex";
            _mutex = new System.Threading.Mutex(true, mutexName, out bool createdNew);
            
            if (!createdNew)
            {
                // Already running
                Shutdown();
                return;
            }

            base.OnStartup(e);
            
            // Context Menu
            var contextMenu = new System.Windows.Controls.ContextMenu();
            
            var openItem = new System.Windows.Controls.MenuItem();
            System.Windows.Data.BindingOperations.SetBinding(openItem, System.Windows.Controls.MenuItem.HeaderProperty, 
                new System.Windows.Data.Binding("[Menu_Open]") { Source = EasyMICBooster.Localization.LocalizationManager.Instance });
            openItem.Click += (_, _) => ShowMainWindow();
            
            var exitItem = new System.Windows.Controls.MenuItem();
            System.Windows.Data.BindingOperations.SetBinding(exitItem, System.Windows.Controls.MenuItem.HeaderProperty, 
                new System.Windows.Data.Binding("[Menu_Exit]") { Source = EasyMICBooster.Localization.LocalizationManager.Instance });
            exitItem.Click += (_, _) => 
            {
                IsExitingFromTray = true;
                _trayIcon?.Dispose();
                Shutdown();
            };

            contextMenu.Items.Add(openItem);
            contextMenu.Items.Add(new System.Windows.Controls.Separator());
            contextMenu.Items.Add(exitItem);
            
            // Create tray icon
            _trayIcon = new TaskbarIcon
            {
                ToolTipText = $"Easy MIC Booster {VersionManager.DisplayVersion}",
                ContextMenu = contextMenu,
                Visibility = Visibility.Visible,
                IconSource = new System.Windows.Media.Imaging.BitmapImage(new Uri("pack://application:,,,/Resources/App.ico"))
            };
            
            // Tray icon setup completed

            // Set up tray icon events
            _trayIcon.TrayLeftMouseDown += (_, _) => ShowMainWindow();
        }

        private void ShowMainWindow()
        {
            if (MainWindow == null)
            {
                MainWindow = new MainWindow();
            }

            MainWindow.Show();
            MainWindow.WindowState = WindowState.Normal;
            MainWindow.Activate();
        }



        protected override void OnExit(ExitEventArgs e)
        {
            _trayIcon?.Dispose();
            base.OnExit(e);
        }
    }
}
