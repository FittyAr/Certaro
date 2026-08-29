using System.Linq;
using ElectroObraApp.Application.DTOs;

namespace ElectroObraApp.ViewModels;

public sealed class CertificadoDocumentItem
{
    public required OrdenTrabajoDto Certificado { get; init; }
    public required TrabajoDto Trabajo { get; init; }

    public string ListTitle => string.IsNullOrWhiteSpace(Certificado.Titulo)
        ? Certificado.NumeroCertificado ?? "—"
        : Certificado.Titulo;

    public string ListSubtitle =>
        $"{Trabajo.Descripcion} · {Certificado.Fecha:dd/MM/yyyy}";

    public decimal TotalActual => Certificado.Items.Sum(i => i.SubtotalActual);

    public decimal TotalAcumulado => Certificado.Items.Sum(i => i.SubtotalAcumulado);
}
