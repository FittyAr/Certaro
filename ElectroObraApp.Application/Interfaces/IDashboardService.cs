using System.Threading.Tasks;
using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.Application.Interfaces;

public interface IDashboardService
{
    Task<DashboardStatsDto> GetStatsAsync(string period);
    Task<bool> CheckDatabaseHealthAsync();
}
