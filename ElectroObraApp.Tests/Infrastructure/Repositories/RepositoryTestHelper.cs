using System.Threading;
using System.Threading.Tasks;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using ElectroObraApp.Infrastructure.Data;
using Xunit;

namespace ElectroObraApp.Tests.Infrastructure.Repositories;

internal static class RepositoryTestHelper
{
    public static async Task<ApplicationDbContext> CreateInMemoryContextAsync()
    {
        var cancellationToken = TestContext.Current.CancellationToken;
        var connection = new SqliteConnection("DataSource=:memory:");
        await connection.OpenAsync(cancellationToken);

        var options = new DbContextOptionsBuilder<ApplicationDbContext>()
            .UseSqlite(connection)
            .Options;

        var context = new ApplicationDbContext(options);
        await context.Database.EnsureCreatedAsync(cancellationToken);

        return context;
    }

    public static CancellationToken CancellationToken => TestContext.Current.CancellationToken;
}
