using System;
using System.IO;
using System.Net;
using System.Net.Mail;
using System.Threading.Tasks;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Infrastructure.Services;

public class SmtpEmailService : IEmailService
{
    private readonly IConfiguration _configuration;
    private readonly ILogger<SmtpEmailService> _logger;

    public SmtpEmailService(IConfiguration configuration, ILogger<SmtpEmailService> logger)
    {
        _configuration = configuration;
        _logger = logger;
    }

    public async Task<bool> SendAsync(string to, string subject, string body, string? attachmentPath = null)
    {
        var host = _configuration["Application:Email:Smtp:Host"];
        var port = _configuration.GetValue("Application:Email:Smtp:Port", 587);
        var user = _configuration["Application:Email:Smtp:User"];
        var password = _configuration["Application:Email:Smtp:Password"];
        var from = _configuration["Application:Email:Smtp:From"] ?? user;
        var enableSsl = _configuration.GetValue("Application:Email:Smtp:EnableSsl", true);

        if (string.IsNullOrWhiteSpace(host) || string.IsNullOrWhiteSpace(from))
        {
            _logger.LogWarning("SMTP no configurado. No se puede enviar email a {To}", to);
            return false;
        }

        try
        {
            using var message = new MailMessage(from, to, subject, body);
            if (!string.IsNullOrWhiteSpace(attachmentPath) && File.Exists(attachmentPath))
            {
                message.Attachments.Add(new Attachment(attachmentPath));
            }

            using var client = new SmtpClient(host, port)
            {
                EnableSsl = enableSsl,
                DeliveryMethod = SmtpDeliveryMethod.Network,
                UseDefaultCredentials = false
            };

            if (!string.IsNullOrWhiteSpace(user))
            {
                client.Credentials = new NetworkCredential(user, password);
            }

            await client.SendMailAsync(message);
            _logger.LogInformation("Email enviado a {To}", to);
            return true;
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error al enviar email a {To}", to);
            return false;
        }
    }
}
