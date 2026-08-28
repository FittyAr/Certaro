using System;
using ElectroObraApp.Core.Interfaces;

namespace ElectroObraApp.Core.Entities;

public abstract class BaseEntity : ISoftDeletable
{
    public Guid Id { get; set; } = Guid.NewGuid();
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
    public DateTime? UpdatedAt { get; set; }
    public byte[] RowVersion { get; set; } = new byte[] { 0, 0, 0, 0, 0, 0, 0, 1 };
    public bool IsDeleted { get; set; }
    public DateTime? DeletedAt { get; set; }
}
