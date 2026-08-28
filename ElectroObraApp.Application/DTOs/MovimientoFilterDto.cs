namespace ElectroObraApp.Application.DTOs;

public class MovimientoFilterDto
{
    public string? Concepto { get; set; }
    public Guid? TipoId { get; set; }
    public DateTime? FechaDesde { get; set; }
    public DateTime? FechaHasta { get; set; }
    public decimal? MontoMin { get; set; }
    public decimal? MontoMax { get; set; }
    public int PageNumber { get; set; } = 1;
    public int PageSize { get; set; } = 10;
}
