using System.Threading.Tasks;

namespace ElectroObraApp.Application.Interfaces;

public interface INotificationService
{
    Task ShowInfoAsync(string title, string message);
    Task ShowWarningAsync(string title, string message);
    Task ShowErrorAsync(string title, string message);
}
