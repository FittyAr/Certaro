using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using ClosedXML.Excel;
using NPOI.XWPF.UserModel;
using QuestPDF.Fluent;
using QuestPDF.Helpers;
using QuestPDF.Infrastructure;
using ElectroObraApp.Application.Interfaces;
using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.Infrastructure.Services;

public class ExportService : IExportService
{
    private readonly IUserSettingsService _settingsService;

    public ExportService(IUserSettingsService settingsService)
    {
        _settingsService = settingsService;
        QuestPDF.Settings.License = LicenseType.Community;
    }

    private string AppName => _settingsService.GetAppName();

    public async Task<byte[]> ExportMovimientosToPdfAsync(IEnumerable<object> movimientos)
    {
        return await Task.Run(() =>
        {
            var data = movimientos.Cast<MovimientoDto>().ToList();
            
            var document = QuestPDF.Fluent.Document.Create(container =>
            {
                container.Page(page =>
                {
                    page.Size(PageSizes.A4);
                    page.Margin(1, Unit.Centimetre);
                    page.PageColor(Colors.White);
                    page.DefaultTextStyle(x => x.FontSize(10));

                    page.Header().Text($"Listado de Movimientos - {AppName}")
                        .SemiBold().FontSize(20).FontColor(Colors.Blue.Medium);

                    page.Content().PaddingVertical(10).Table(table =>
                    {
                        table.ColumnsDefinition(columns =>
                        {
                            columns.RelativeColumn(2); // Fecha
                            columns.RelativeColumn(4); // Concepto
                            columns.RelativeColumn(2); // Tipo
                            columns.RelativeColumn(2); // Monto
                        });

                        table.Header(header =>
                        {
                            header.Cell().Element(CellStyle).Text("Fecha");
                            header.Cell().Element(CellStyle).Text("Concepto");
                            header.Cell().Element(CellStyle).Text("Tipo");
                            header.Cell().Element(CellStyle).Text("Monto");

                            static IContainer CellStyle(IContainer container)
                            {
                                return container.DefaultTextStyle(x => x.SemiBold())
                                    .PaddingVertical(5).BorderBottom(1).BorderColor(Colors.Black);
                            }
                        });

                        foreach (var item in data)
                        {
                            table.Cell().Element(ValueStyle).Text(item.Fecha.ToString("dd/MM/yyyy"));
                            table.Cell().Element(ValueStyle).Text(item.Concepto);
                            table.Cell().Element(ValueStyle).Text(item.TipoMovimientoNombre);
                            table.Cell().Element(ValueStyle).Text(item.Total.ToString("C"));

                            static IContainer ValueStyle(IContainer container)
                            {
                                return container.PaddingVertical(5).BorderBottom(1).BorderColor(Colors.Grey.Lighten2);
                            }
                        }
                    });

                    page.Footer().AlignCenter().Text(x =>
                    {
                        x.Span("Página ");
                        x.CurrentPageNumber();
                    });
                });
            });

            return document.GeneratePdf();
        });
    }

    public async Task<byte[]> ExportMovimientosToExcelAsync(IEnumerable<object> movimientos)
    {
        return await Task.Run(() =>
        {
            var data = movimientos.Cast<MovimientoDto>().ToList();
            using var workbook = new XLWorkbook();
            var worksheet = workbook.Worksheets.Add("Movimientos");

            // Cabeceras
            worksheet.Cell(1, 1).Value = "Fecha";
            worksheet.Cell(1, 2).Value = "Concepto";
            worksheet.Cell(1, 3).Value = "Tipo";
            worksheet.Cell(1, 4).Value = "Monto";
            worksheet.Cell(1, 5).Value = "Cantidad";
            worksheet.Cell(1, 6).Value = "Total";

            var headerRow = worksheet.Row(1);
            headerRow.Style.Font.Bold = true;
            headerRow.Style.Fill.BackgroundColor = XLColor.LightBlue;

            // Datos
            for (int i = 0; i < data.Count; i++)
            {
                var item = data[i];
                var row = i + 2;
                worksheet.Cell(row, 1).Value = item.Fecha;
                worksheet.Cell(row, 2).Value = item.Concepto;
                worksheet.Cell(row, 3).Value = item.TipoMovimientoNombre;
                worksheet.Cell(row, 4).Value = item.Monto;
                worksheet.Cell(row, 5).Value = item.Cantidad;
                worksheet.Cell(row, 6).Value = item.Total;
            }

            worksheet.Columns().AdjustToContents();

            using var stream = new MemoryStream();
            workbook.SaveAs(stream);
            return stream.ToArray();
        });
    }

    public async Task<byte[]> ExportMovimientosToWordAsync(IEnumerable<object> movimientos)
    {
        return await Task.Run(() =>
        {
            var data = movimientos.Cast<MovimientoDto>().ToList();
            var doc = new XWPFDocument();

            var title = doc.CreateParagraph();
            title.Alignment = ParagraphAlignment.CENTER;
            var titleRun = title.CreateRun();
            titleRun.SetText($"Listado de Movimientos - {AppName}");
            titleRun.FontSize = 20;
            titleRun.IsBold = true;

            var table = doc.CreateTable(data.Count + 1, 4);
            table.Width = 5000;

            // Cabeceras
            var headerRow = table.GetRow(0);
            headerRow.GetCell(0).SetText("Fecha");
            headerRow.GetCell(1).SetText("Concepto");
            headerRow.GetCell(2).SetText("Tipo");
            headerRow.GetCell(3).SetText("Total");

            // Datos
            for (int i = 0; i < data.Count; i++)
            {
                var item = data[i];
                var row = table.GetRow(i + 1);
                row.GetCell(0).SetText(item.Fecha.ToString("dd/MM/yyyy"));
                row.GetCell(1).SetText(item.Concepto);
                row.GetCell(2).SetText(item.TipoMovimientoNombre);
                row.GetCell(3).SetText(item.Total.ToString("C"));
            }

            using var stream = new MemoryStream();
            doc.Write(stream);
            return stream.ToArray();
        });
    }

    public async Task<byte[]> ExportMovimientosToCsvAsync(IEnumerable<object> movimientos)
    {
        return await Task.Run(() =>
        {
            var data = movimientos.Cast<MovimientoDto>().ToList();
            using var stream = new MemoryStream();
            using var writer = new StreamWriter(stream);
            writer.WriteLine("Fecha,Concepto,Tipo,Monto,Cantidad,Total");
            foreach (var item in data)
            {
                writer.WriteLine(string.Join(",",
                    EscapeCsv(item.Fecha.ToString("dd/MM/yyyy")),
                    EscapeCsv(item.Concepto),
                    EscapeCsv(item.TipoMovimientoNombre),
                    item.Monto.ToString(System.Globalization.CultureInfo.InvariantCulture),
                    item.Cantidad.ToString(System.Globalization.CultureInfo.InvariantCulture),
                    item.Total.ToString(System.Globalization.CultureInfo.InvariantCulture)));
            }
            writer.Flush();
            return stream.ToArray();
        });
    }

    public async Task<byte[]> ExportMovimientosToJsonAsync(IEnumerable<object> movimientos)
    {
        return await Task.Run(() =>
        {
            var json = System.Text.Json.JsonSerializer.Serialize(movimientos, new System.Text.Json.JsonSerializerOptions { WriteIndented = true });
            return System.Text.Encoding.UTF8.GetBytes(json);
        });
    }

    public async Task<byte[]> ExportLiquidacionToPdfAsync(LiquidacionDto liquidacion, IEnumerable<MovimientoDto> adelantos)
    {
        return await Task.Run(() =>
        {
            var document = QuestPDF.Fluent.Document.Create(container =>
            {
                container.Page(page =>
                {
                    page.Size(PageSizes.A4);
                    page.Margin(1.5f, Unit.Centimetre);
                    page.PageColor(Colors.White);
                    page.DefaultTextStyle(x => x.FontSize(11).FontColor(Colors.Grey.Darken3));

                    // Header
                    page.Header().Row(row =>
                    {
                        row.RelativeItem().Column(col =>
                        {
                            col.Item().Text("REPORTE DE LIQUIDACIÓN").SemiBold().FontSize(24).FontColor(Colors.Blue.Medium);
                            col.Item().Text($"Empleado: {liquidacion.EmpleadoNombre}").FontSize(14).SemiBold();
                            col.Item().Text($"Periodo: {liquidacion.FechaInicio:dd/MM/yyyy} al {liquidacion.FechaFin:dd/MM/yyyy}").FontSize(10);
                        });

                        row.RelativeItem().AlignRight().Column(col =>
                        {
                            col.Item().Text(AppName.ToUpperInvariant()).SemiBold().FontSize(12);
                            col.Item().Text("Cuentas Claras").Italic().FontSize(10);
                            col.Item().Text(DateTime.Now.ToString("dd/MM/yyyy HH:mm")).FontSize(8);
                        });
                    });

                    page.Content().PaddingVertical(20).Column(col =>
                    {
                        // Resumen de Trabajo
                        col.Item().PaddingBottom(10).Text("RESUMEN DE TRABAJO").SemiBold().Underline();
                        col.Item().Table(table =>
                        {
                            table.ColumnsDefinition(columns =>
                            {
                                columns.RelativeColumn(3);
                                columns.RelativeColumn(1);
                                columns.RelativeColumn(2);
                                columns.RelativeColumn(2);
                            });

                            table.Header(header =>
                            {
                                header.Cell().Element(HStyle).Text("Concepto");
                                header.Cell().AlignRight().Element(HStyle).Text("Días");
                                header.Cell().AlignRight().Element(HStyle).Text("Tarifa");
                                header.Cell().AlignRight().Element(HStyle).Text("Subtotal");
                            });

                            table.Cell().Element(VStyle).Text("Días trabajados en el periodo");
                            table.Cell().AlignRight().Element(VStyle).Text(liquidacion.DiasTrabajados.ToString("N1"));
                            table.Cell().AlignRight().Element(VStyle).Text(liquidacion.TarifaAplicada.ToString("C"));
                            table.Cell().AlignRight().Element(VStyle).Text(liquidacion.TotalBruto.ToString("C")).SemiBold();
                        });

                        // Detalle de Adelantos
                        col.Item().PaddingTop(20).PaddingBottom(10).Text("DETALLE DE ADELANTOS (DESCUENTOS)").SemiBold().Underline().FontColor(Colors.Red.Medium);
                        col.Item().Table(table =>
                        {
                            table.ColumnsDefinition(columns =>
                            {
                                columns.RelativeColumn(2);
                                columns.RelativeColumn(4);
                                columns.RelativeColumn(2);
                            });

                            table.Header(header =>
                            {
                                header.Cell().Element(HStyle).Text("Fecha");
                                header.Cell().Element(HStyle).Text("Concepto");
                                header.Cell().AlignRight().Element(HStyle).Text("Monto");
                            });

                            foreach (var item in adelantos)
                            {
                                table.Cell().Element(VStyle).Text(item.Fecha.ToString("dd/MM/yyyy"));
                                table.Cell().Element(VStyle).Text(item.Concepto);
                                table.Cell().AlignRight().Element(VStyle).Text(item.Total.ToString("C"));
                            }

                            if (!adelantos.Any())
                            {
                                table.Cell().ColumnSpan(3).Element(VStyle).AlignCenter().Text("No se registraron adelantos en este periodo").Italic();
                            }

                            table.Footer(footer =>
                            {
                                footer.Cell().ColumnSpan(2).PaddingVertical(5).AlignRight().Text("TOTAL ADELANTOS:").SemiBold();
                                footer.Cell().PaddingVertical(5).AlignRight().Text(liquidacion.TotalAdelantos.ToString("C")).SemiBold().FontColor(Colors.Red.Medium);
                            });
                        });

                        // Total Final
                        col.Item().PaddingTop(30).AlignRight().Container().Width(200).Background(Colors.Grey.Lighten4).Padding(10).Column(totalCol =>
                        {
                            totalCol.Item().Row(row =>
                            {
                                row.RelativeItem().Text("Subtotal:");
                                row.RelativeItem().AlignRight().Text(liquidacion.TotalBruto.ToString("C"));
                            });
                            totalCol.Item().Row(row =>
                            {
                                row.RelativeItem().Text("Adelantos:");
                                row.RelativeItem().AlignRight().Text($"- {liquidacion.TotalAdelantos:C}").FontColor(Colors.Red.Medium);
                            });
                            totalCol.Item().PaddingTop(5).BorderTop(1).Row(row =>
                            {
                                row.RelativeItem().Text("TOTAL A PAGAR:").SemiBold().FontSize(14);
                                row.RelativeItem().AlignRight().Text(liquidacion.TotalNeto.ToString("C")).SemiBold().FontSize(14).FontColor(Colors.Green.Medium);
                            });
                        });

                        if (!string.IsNullOrEmpty(liquidacion.Observaciones))
                        {
                            col.Item().PaddingTop(20).Text(x =>
                            {
                                x.Span("Observaciones: ").SemiBold();
                                x.Span(liquidacion.Observaciones);
                            });
                        }

                        // Firma
                        col.Item().PaddingTop(60).Row(row =>
                        {
                            row.RelativeItem().Column(c =>
                            {
                                c.Item().PaddingTop(40).BorderTop(1).AlignCenter().Text("Firma de Revisión");
                            });
                            row.ConstantItem(100);
                            row.RelativeItem().Column(c =>
                            {
                                c.Item().PaddingTop(40).BorderTop(1).AlignCenter().Text("Generado por Administración");
                            });
                        });
                    });

                    static IContainer HStyle(IContainer container) => container.DefaultTextStyle(x => x.SemiBold()).PaddingVertical(5).BorderBottom(1).BorderColor(Colors.Black);
                    static IContainer VStyle(IContainer container) => container.PaddingVertical(5).BorderBottom(1).BorderColor(Colors.Grey.Lighten2);

                    page.Footer().AlignCenter().Text(x =>
                    {
                        x.Span($"{AppName} - Software de Gestión Profesional - Página ");
                        x.CurrentPageNumber();
                    });
                });
            });

            return document.GeneratePdf();
        });
    }

    public async Task<byte[]> ExportCertificadoToPdfAsync(OrdenTrabajoDto certificado, TrabajoDto trabajo)
    {
        return await Task.Run(() =>
        {
            var document = QuestPDF.Fluent.Document.Create(container =>
            {
                container.Page(page =>
                {
                    page.Size(PageSizes.A4.Landscape()); // Paisaje para que entren todas las columnas de la foto
                    page.Margin(1, Unit.Centimetre);
                    page.PageColor(Colors.White);
                    page.DefaultTextStyle(x => x.FontSize(9).FontColor(Colors.Black));

                    // Header (Estilo Foto 1/3)
                    page.Header().Border(1).Row(row =>
                    {
                        row.RelativeItem(3).Padding(5).Column(col =>
                        {
                            col.Item().Row(r => { r.ConstantItem(50).Text("Obra:").SemiBold(); r.RelativeItem().Text(trabajo.Id != Guid.Empty ? trabajo.Descripcion : "Obra General"); });
                            col.Item().Row(r => { r.ConstantItem(50).Text("Ref:").SemiBold(); r.RelativeItem().Text(certificado.Titulo); });
                            col.Item().Row(r => { r.ConstantItem(60).Text("Contratista:").SemiBold(); r.RelativeItem().Text("PABLO BAEZ"); });
                        });

                        row.RelativeItem(2).BorderLeft(1).Padding(5).Column(col =>
                        {
                            col.Item().AlignCenter().Text("GENERCON").FontSize(16).Black().SemiBold();
                            col.Item().AlignCenter().Text("ENERGIA CONTROLADA").FontSize(8);
                            col.Item().PaddingTop(5).Row(r => { r.RelativeItem().Text("FECHA:").SemiBold(); r.RelativeItem().AlignRight().Text(certificado.Fecha.ToString("dd/MM/yyyy")); });
                            col.Item().Row(r => { r.RelativeItem().Text("CERTIFICADO N°:").SemiBold(); r.RelativeItem().AlignRight().Text(certificado.NumeroCertificado ?? "1"); });
                        });
                    });

                    page.Content().PaddingVertical(10).Table(table =>
                    {
                        table.ColumnsDefinition(columns =>
                        {
                            columns.RelativeColumn(3); // Descripción
                            columns.ConstantColumn(30); // UND
                            columns.ConstantColumn(40); // Cant
                            columns.ConstantColumn(70); // P.U.
                            columns.ConstantColumn(50); // Ant %
                            columns.ConstantColumn(50); // Act %
                            columns.ConstantColumn(50); // Acu %
                            columns.ConstantColumn(80); // Importe Act
                            columns.ConstantColumn(80); // Importe Acu
                        });

                        table.Header(header =>
                        {
                            header.Cell().RowSpan(2).Element(HStyle).Text("ITEM / DESCRIPCIÓN");
                            header.Cell().ColumnSpan(2).Element(HStyle).AlignCenter().Text("CÓMPUTOS");
                            header.Cell().RowSpan(2).Element(HStyle).AlignCenter().Text("P.U.");
                            header.Cell().ColumnSpan(3).Element(HStyle).AlignCenter().Text("AVANCE (%)");
                            header.Cell().ColumnSpan(2).Element(HStyle).AlignCenter().Text("IMPORTE");

                            header.Cell().Element(HStyleSub).AlignCenter().Text("UND");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("CANT");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("ANT");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("ACT");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("ACU");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("ACTUAL");
                            header.Cell().Element(HStyleSub).AlignCenter().Text("ACUMULADO");

                            static IContainer HStyle(IContainer container) => container.Background(Colors.Green.Darken2).PaddingVertical(2).Border(0.5f).BorderColor(Colors.Black).AlignCenter().DefaultTextStyle(x => x.SemiBold().FontColor(Colors.White).FontSize(8));
                            static IContainer HStyleSub(IContainer container) => container.Background(Colors.Green.Lighten1).PaddingVertical(1).Border(0.5f).BorderColor(Colors.Black).AlignCenter().DefaultTextStyle(x => x.FontSize(7));
                        });

                        decimal totalActual = 0;
                        decimal totalAcumulado = 0;

                        foreach (var item in certificado.Items)
                        {
                            table.Cell().Element(VStyle).Text(item.Descripcion);
                            table.Cell().Element(VStyle).AlignCenter().Text(item.Unidad);
                            table.Cell().Element(VStyle).AlignCenter().Text(item.Cantidad.ToString("N0"));
                            table.Cell().Element(VStyle).AlignRight().Text(item.PrecioUnitario.ToString("C2"));
                            
                            table.Cell().Element(VStyle).AlignCenter().Text($"{item.PorcentajeAnterior:N1}%");
                            table.Cell().Element(VStyle).AlignCenter().Text($"{item.PorcentajeActual:N1}%").SemiBold();
                            table.Cell().Element(VStyle).AlignCenter().Text($"{item.PorcentajeAcumulado:N1}%");

                            table.Cell().Element(VStyle).AlignRight().Text(item.SubtotalActual.ToString("C2"));
                            table.Cell().Element(VStyle).AlignRight().Text(item.SubtotalAcumulado.ToString("C2"));

                            totalActual += item.SubtotalActual;
                            totalAcumulado += item.SubtotalAcumulado;
                        }

                        // Footer de Totales y Ajustes
                        table.Footer(footer =>
                        {
                            footer.Cell().ColumnSpan(7).Padding(2).AlignRight().Text("SUB-TOTAL:").SemiBold();
                            footer.Cell().Padding(2).AlignRight().Text(totalActual.ToString("C2")).SemiBold();
                            footer.Cell().Padding(2).AlignRight().Text(totalAcumulado.ToString("C2")).SemiBold();

                            if (certificado.AjusteUocraPorcentaje > 0)
                            {
                                var ajuste = totalActual * (certificado.AjusteUocraPorcentaje / 100);
                                footer.Cell().ColumnSpan(7).Padding(2).AlignRight().Text($"AJUSTE UOCRA {certificado.AjusteUocraPorcentaje:N0}%:").Italic();
                                footer.Cell().Padding(2).AlignRight().Text(ajuste.ToString("C2"));
                                footer.Cell().Padding(2).AlignRight().Text("");
                                totalActual += ajuste;
                            }

                            if (certificado.OtrosDescuentos > 0)
                            {
                                footer.Cell().ColumnSpan(7).Padding(2).AlignRight().Text("OTROS DESCUENTOS:").Italic();
                                footer.Cell().Padding(2).AlignRight().Text($"-{certificado.OtrosDescuentos:C2}");
                                footer.Cell().Padding(2).AlignRight().Text("");
                                totalActual -= certificado.OtrosDescuentos;
                            }

                            footer.Cell().ColumnSpan(7).Background(Colors.Green.Lighten4).Padding(4).AlignRight().Text("TOTAL A FACTURAR:").FontSize(11).SemiBold();
                            footer.Cell().Background(Colors.Green.Lighten4).Padding(4).AlignRight().Text(totalActual.ToString("C2")).FontSize(11).SemiBold();
                            footer.Cell().Background(Colors.Green.Lighten4).Padding(4).AlignRight().Text("");
                        });

                        static IContainer VStyle(IContainer container) => container.Border(0.5f).BorderColor(Colors.Black).PaddingHorizontal(3).PaddingVertical(2);
                    });

                    page.Footer().AlignCenter().Text(x =>
                    {
                        x.Span($"Certificado generado por {AppName} - ");
                        x.CurrentPageNumber();
                    });
                });
            });

            return document.GeneratePdf();
        });
    }

    public static string EscapeCsv(string? value)
    {
        if (string.IsNullOrEmpty(value))
            return string.Empty;

        var needsQuotes = value.Contains(',') || value.Contains('"') || value.Contains('\n') || value.Contains('\r');
        var escaped = value.Replace("\"", "\"\"");
        return needsQuotes ? $"\"{escaped}\"" : escaped;
    }
}

