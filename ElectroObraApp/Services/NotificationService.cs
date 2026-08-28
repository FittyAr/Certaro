using System;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Services;

public class NotificationService : INotificationService
{
    private WindowNotificationManager? _manager;

    private WindowNotificationManager GetManager()
    {
        if (_manager is not null)
            return _manager;

        if (global::Avalonia.Application.Current?.ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop &&
            desktop.MainWindow is Window window)
        {
            _manager = new WindowNotificationManager(window);
        }

        return _manager ?? throw new InvalidOperationException("No se pudo inicializar WindowNotificationManager.");
    }

    public Task ShowInfoAsync(string title, string message) =>
        ShowAsync(title, message, NotificationType.Information);

    public Task ShowWarningAsync(string title, string message) =>
        ShowAsync(title, message, NotificationType.Warning);

    public Task ShowErrorAsync(string title, string message) =>
        ShowAsync(title, message, NotificationType.Error);

    private Task ShowAsync(string title, string message, NotificationType type)
    {
        return Dispatcher.UIThread.InvokeAsync(() =>
        {
            GetManager().Show(new Notification(title, message, type));
        }).GetTask();
    }
}
