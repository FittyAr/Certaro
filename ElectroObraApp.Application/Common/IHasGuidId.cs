using System;

namespace ElectroObraApp.Application.Common;

public interface IHasGuidId
{
    Guid Id { get; set; }
}
