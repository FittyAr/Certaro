using System.Threading.Tasks;

namespace ElectroObraApp.Application.Interfaces;

public interface IFileSaveDialogService
{
    Task<bool> SaveFileAsync(byte[] content, string suggestedFileName, string extension);
}
