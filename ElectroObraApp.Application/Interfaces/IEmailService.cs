using System.Threading.Tasks;

namespace ElectroObraApp.Application.Interfaces;

public interface IEmailService
{
    Task<bool> SendAsync(string to, string subject, string body, string? attachmentPath = null);
}
