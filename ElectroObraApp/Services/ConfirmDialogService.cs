using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Services;

public class ConfirmDialogService : IConfirmDialogService
{
    public async Task<bool> ConfirmAsync(string title, string message)
    {
        if (global::Avalonia.Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop ||
            desktop.MainWindow is null)
        {
            return false;
        }

        var tcs = new TaskCompletionSource<bool>();

        await Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var dialog = new Window
            {
                Title = title,
                Width = 420,
                Height = 200,
                WindowStartupLocation = WindowStartupLocation.CenterOwner,
                CanResize = false,
                Background = new SolidColorBrush(Color.Parse("#2d2d2d"))
            };

            var panel = new StackPanel { Margin = new Thickness(20), Spacing = 15 };
            panel.Children.Add(new TextBlock
            {
                Text = message,
                TextWrapping = TextWrapping.Wrap,
                Foreground = Brushes.White
            });

            var buttons = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                HorizontalAlignment = HorizontalAlignment.Right,
                Spacing = 10
            };

            var cancelButton = new Button { Content = "Cancelar", MinWidth = 90 };
            var confirmButton = new Button { Content = "Eliminar", MinWidth = 90, Classes = { "accent" } };

            cancelButton.Click += (_, _) =>
            {
                tcs.TrySetResult(false);
                dialog.Close();
            };
            confirmButton.Click += (_, _) =>
            {
                tcs.TrySetResult(true);
                dialog.Close();
            };

            buttons.Children.Add(cancelButton);
            buttons.Children.Add(confirmButton);
            panel.Children.Add(buttons);
            dialog.Content = panel;

            dialog.Closed += (_, _) => tcs.TrySetResult(false);
            await dialog.ShowDialog(desktop.MainWindow);
        });

        return await tcs.Task;
    }
}
