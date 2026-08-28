using System.Linq.Expressions;
using ElectroObraApp.Core.Entities;

namespace ElectroObraApp.Core.Specifications;

public class MovimientosPagedSpecification : BaseSpecification<Movimiento>
{
    public MovimientosPagedSpecification(
        string? concepto,
        Guid? tipoId,
        DateTime? fechaDesde,
        DateTime? fechaHasta,
        decimal? montoMin,
        decimal? montoMax,
        int pageNumber,
        int pageSize)
        : base(BuildCriteria(concepto, tipoId, fechaDesde, fechaHasta, montoMin, montoMax))
    {
        AddInclude(m => m.TipoMovimiento);
        AddInclude(m => m.Categoria);
        ApplyOrderByDescending(m => (object)m.Fecha);

        if (pageSize > 0)
            ApplyPaging((pageNumber - 1) * pageSize, pageSize);
    }

    private static Expression<Func<Movimiento, bool>>? BuildCriteria(
        string? concepto,
        Guid? tipoId,
        DateTime? fechaDesde,
        DateTime? fechaHasta,
        decimal? montoMin,
        decimal? montoMax)
    {
        Expression<Func<Movimiento, bool>>? criteria = null;

        if (!string.IsNullOrWhiteSpace(concepto))
        {
            var term = concepto.ToLower();
            Expression<Func<Movimiento, bool>> predicate = m => m.Concepto.ToLower().Contains(term);
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        if (tipoId.HasValue)
        {
            var id = tipoId.Value;
            Expression<Func<Movimiento, bool>> predicate = m => m.TipoMovimientoId == id;
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        if (fechaDesde.HasValue)
        {
            var date = fechaDesde.Value.Date;
            Expression<Func<Movimiento, bool>> predicate = m => m.Fecha.Date >= date;
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        if (fechaHasta.HasValue)
        {
            var date = fechaHasta.Value.Date;
            Expression<Func<Movimiento, bool>> predicate = m => m.Fecha.Date <= date;
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        if (montoMin.HasValue)
        {
            var min = montoMin.Value;
            Expression<Func<Movimiento, bool>> predicate = m => m.Monto >= min;
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        if (montoMax.HasValue)
        {
            var max = montoMax.Value;
            Expression<Func<Movimiento, bool>> predicate = m => m.Monto <= max;
            criteria = criteria is null ? predicate : criteria.And(predicate);
        }

        return criteria;
    }
}
