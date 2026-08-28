using System;
using Serilog.Core;
using Serilog.Events;

namespace ElectroObraApp.Infrastructure.Services;

/// <summary>
/// Stub sink prepared for future centralized audit logging in SQLite/SQL Server.
/// Enable from <see cref="SerilogConfiguration"/> once the audit schema is defined.
/// </summary>
public sealed class SerilogDbSink : ILogEventSink
{
    public void Emit(LogEvent logEvent)
    {
        // TODO: Persist logEvent to audit table (Level, Message, Exception, Timestamp, MachineName, ThreadId).
        _ = logEvent;
    }
}
