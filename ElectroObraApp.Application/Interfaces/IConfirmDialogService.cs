using System.Threading.Tasks;

namespace ElectroObraApp.Application.Interfaces;

public interface IConfirmDialogService
{
    Task<bool> ConfirmAsync(string title, string message);
}
