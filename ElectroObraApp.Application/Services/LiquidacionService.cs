using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Mapster;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.Validation;
using ElectroObraApp.Core.Common;
using ElectroObraApp.Core.Entities;
using ElectroObraApp.Core.Enums;
using ElectroObraApp.Core.Interfaces;
using FluentValidation;

namespace ElectroObraApp.Application.Services;

public class LiquidacionService : BaseCrudService<Liquidacion, LiquidacionDto>, ILiquidacionService
{
    private readonly IUserSettingsService _settingsService;
    private readonly IHolidayService _holidayService;

    public LiquidacionService(
        IUnitOfWork uow,
        ILogger<LiquidacionService> logger,
        IValidator<LiquidacionDto> validator,
        IUserSettingsService settingsService,
        IHolidayService holidayService)
        : base(uow, logger, validator)
    {
        _settingsService = settingsService;
        _holidayService = holidayService;
    }

    public async Task<IEnumerable<LiquidacionDto>> GetAllAsync()
    {
        var entities = await Uow.Liquidaciones.GetAllWithEmpleadoAsync();
        return entities.Adapt<IEnumerable<LiquidacionDto>>();
    }

    public async Task<LiquidacionDto?> GetByIdAsync(Guid id)
    {
        var entity = await Repository.GetByIdAsync(id);
        return entity?.Adapt<LiquidacionDto>();
    }

    public new async Task<Result<LiquidacionDto>> CreateAsync(LiquidacionDto dto)
    {
        var validation = await ValidateAsync(dto);
        if (!validation.IsSuccess)
            return Result<LiquidacionDto>.Failure(validation.Errors);

        Logger.LogInformation("Creando liquidación para empleado: {EmpleadoId}", dto.EmpleadoId);
        var entity = dto.Adapt<Liquidacion>();
        await Repository.AddAsync(entity);

        if (await Uow.SaveChangesAsync() <= 0)
            return Result<LiquidacionDto>.Failure(ValidationMessages.SaveFailed);

        return Result<LiquidacionDto>.Success(entity.Adapt<LiquidacionDto>());
    }

    public async Task<Result<IReadOnlyList<LiquidacionDto>>> CreateBatchAsync(IEnumerable<LiquidacionDto> dtos)
    {
        var list = dtos.ToList();
        if (list.Count == 0)
            return Result<IReadOnlyList<LiquidacionDto>>.Failure(ValidationMessages.LiquidacionBatchEmpty);

        await Uow.BeginTransactionAsync();
        try
        {
            var created = new List<LiquidacionDto>();

            foreach (var dto in list)
            {
                var validation = await ValidateAsync(dto);
                if (!validation.IsSuccess)
                {
                    await Uow.RollbackTransactionAsync();
                    return Result<IReadOnlyList<LiquidacionDto>>.Failure(validation.Errors);
                }

                Logger.LogInformation("Creando liquidación batch para empleado: {EmpleadoId}", dto.EmpleadoId);
                var entity = dto.Adapt<Liquidacion>();
                await Repository.AddAsync(entity);
                created.Add(entity.Adapt<LiquidacionDto>());
            }

            if (await Uow.SaveChangesAsync() <= 0)
            {
                await Uow.RollbackTransactionAsync();
                return Result<IReadOnlyList<LiquidacionDto>>.Failure(ValidationMessages.SaveFailed);
            }

            await Uow.CommitTransactionAsync();
            return Result<IReadOnlyList<LiquidacionDto>>.Success(created);
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Error al crear liquidaciones en batch");
            await Uow.RollbackTransactionAsync();
            throw;
        }
    }

    public async Task<LiquidacionDto> SugerirLiquidacionAsync(Guid empleadoId, DateTime inicio, DateTime fin, decimal diasTrabajados)
    {
        var empleado = await Uow.Repository<Empleado>().GetByIdAsync(empleadoId);
        if (empleado is null)
            throw new InvalidOperationException(ValidationMessages.EntityNotFound);

        var incluirSabados = _settingsService.GetDefaultIncludeSaturday();
        var incluirDomingos = _settingsService.GetDefaultIncludeSunday();
        var incluirFeriados = _settingsService.GetDefaultIncludeHoliday();
        var multiplicadorSabado = _settingsService.GetDefaultMultiplierSaturday();
        var multiplicadorDomingo = _settingsService.GetDefaultMultiplierSunday();
        var multiplicadorFeriado = _settingsService.GetDefaultMultiplierHoliday();

        var feriados = await ObtenerFeriadosAsync(inicio, fin);

        decimal totalDias;
        decimal totalBruto;

        if (diasTrabajados == 0)
        {
            var calculoAsistencia = await CalcularDesdeAsistenciaAsync(
                empleadoId,
                inicio,
                fin,
                empleado,
                feriados,
                incluirSabados,
                incluirDomingos,
                incluirFeriados,
                multiplicadorSabado,
                multiplicadorDomingo,
                multiplicadorFeriado);

            if (calculoAsistencia.HasValue)
            {
                totalDias = calculoAsistencia.Value.Dias;
                totalBruto = calculoAsistencia.Value.Bruto;
            }
            else
            {
                totalDias = 0;
                totalBruto = 0;

                for (var date = inicio.Date; date <= fin.Date; date = date.AddDays(1))
                {
                    var multiplicador = ObtenerMultiplicador(
                        date,
                        feriados,
                        incluirSabados,
                        incluirDomingos,
                        incluirFeriados,
                        multiplicadorSabado,
                        multiplicadorDomingo,
                        multiplicadorFeriado);

                    if (multiplicador <= 0) continue;

                    totalDias += 1.0m;
                    totalBruto += empleado.TarifaDiaria * multiplicador;
                }
            }
        }
        else
        {
            totalDias = diasTrabajados;
            totalBruto = totalDias * empleado.TarifaDiaria;
        }

        var adelantoTypeId = Core.Constants.TiposMovimiento.Adelanto;
        var movimientos = await Uow.Movimientos.FindAsync(m =>
            m.EmpleadoId == empleadoId &&
            m.Fecha >= inicio &&
            m.Fecha <= fin &&
            m.TipoMovimientoId == adelantoTypeId);

        var totalAdelantos = movimientos.Sum(m => m.Total);

        Logger.LogInformation(
            "Liquidación sugerida para empleado {EmpleadoId}: {Dias} días, bruto {Bruto}, adelantos {Adelantos}, feriados {Feriados}",
            empleadoId,
            totalDias,
            totalBruto,
            totalAdelantos,
            feriados.Count);

        return new LiquidacionDto
        {
            EmpleadoId = empleadoId,
            EmpleadoNombre = empleado.Nombre,
            FechaInicio = inicio,
            FechaFin = fin,
            DiasTrabajados = totalDias,
            TarifaAplicada = empleado.TarifaDiaria,
            TotalAdelantos = totalAdelantos,
            TotalBruto = totalBruto,
            TotalNeto = totalBruto - totalAdelantos,
            IncluirSabados = incluirSabados,
            IncluirDomingos = incluirDomingos,
            IncluirFeriados = incluirFeriados,
            MultiplicadorSabado = multiplicadorSabado,
            MultiplicadorDomingo = multiplicadorDomingo,
            MultiplicadorFeriado = multiplicadorFeriado
        };
    }

    private async Task<HashSet<DateTime>> ObtenerFeriadosAsync(DateTime inicio, DateTime fin)
    {
        var feriados = new HashSet<DateTime>();

        for (var year = inicio.Year; year <= fin.Year; year++)
        {
            var holidays = await _holidayService.GetHolidaysAsync(year);
            foreach (var holiday in holidays)
            {
                feriados.Add(holiday.Date.Date);
            }
        }

        return feriados;
    }

    private static decimal ObtenerMultiplicador(
        DateTime date,
        HashSet<DateTime> feriados,
        bool incluirSabados,
        bool incluirDomingos,
        bool incluirFeriados,
        decimal multiplicadorSabado,
        decimal multiplicadorDomingo,
        decimal multiplicadorFeriado)
    {
        var esSabado = date.DayOfWeek == DayOfWeek.Saturday;
        var esDomingo = date.DayOfWeek == DayOfWeek.Sunday;
        var esFeriado = feriados.Contains(date.Date);

        if (esFeriado)
            return incluirFeriados ? multiplicadorFeriado : 0.0m;

        if (esDomingo)
            return incluirDomingos ? multiplicadorDomingo : 0.0m;

        if (esSabado)
            return incluirSabados ? multiplicadorSabado : 0.0m;

        return 1.0m;
    }

    private async Task<(decimal Dias, decimal Bruto)?> CalcularDesdeAsistenciaAsync(
        Guid empleadoId,
        DateTime inicio,
        DateTime fin,
        Empleado empleado,
        HashSet<DateTime> feriados,
        bool incluirSabados,
        bool incluirDomingos,
        bool incluirFeriados,
        decimal multiplicadorSabado,
        decimal multiplicadorDomingo,
        decimal multiplicadorFeriado)
    {
        var asistencias = await Uow.Repository<AsistenciaEmpleado>().FindAsync(a =>
            a.EmpleadoId == empleadoId &&
            a.Fecha >= inicio.Date &&
            a.Fecha <= fin.Date);

        var registros = asistencias.ToList();
        if (registros.Count == 0)
            return null;

        decimal totalDias = 0;
        decimal totalBruto = 0;

        foreach (var asistencia in registros)
        {
            var factor = ObtenerFactorJornada(asistencia.TipoJornada);
            if (factor <= 0) continue;

            var multiplicador = asistencia.TipoJornada == TipoJornada.Feriado
                ? (incluirFeriados ? multiplicadorFeriado : 0.0m)
                : ObtenerMultiplicador(
                    asistencia.Fecha,
                    feriados,
                    incluirSabados,
                    incluirDomingos,
                    incluirFeriados,
                    multiplicadorSabado,
                    multiplicadorDomingo,
                    multiplicadorFeriado);

            if (multiplicador <= 0) continue;

            totalDias += factor;
            totalBruto += empleado.TarifaDiaria * factor * multiplicador;
        }

        return (totalDias, totalBruto);
    }

    private static decimal ObtenerFactorJornada(TipoJornada tipoJornada) =>
        tipoJornada switch
        {
            TipoJornada.Completa => 1.0m,
            TipoJornada.Media => 0.5m,
            TipoJornada.Feriado => 1.0m,
            _ => 0.0m
        };
}
