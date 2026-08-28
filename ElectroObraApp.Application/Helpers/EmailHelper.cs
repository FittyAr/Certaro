using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Application.Helpers;

public static class EmailHelper
{
    public static void OpenEmailClient(string email, IUserSettingsService settingsService, string? subject = null, string? body = null)
    {
        if (string.IsNullOrWhiteSpace(email)) return;

        var encodedEmail = Uri.EscapeDataString(email);
        var client = settingsService.GetPreferredEmailClient();
        string url = client switch
        {
            "Gmail" => BuildWebClientUrl(settingsService.GetGmailUrl(), email, encodedEmail, subject, body),
            "Yahoo" => BuildWebClientUrl(settingsService.GetYahooUrl(), email, encodedEmail, subject, body),
            "OutlookWeb" => BuildWebClientUrl(settingsService.GetOutlookUrl(), email, encodedEmail, subject, body),
            _ => BuildMailtoUrl(email, subject, body)
        };

        OpenUrl(url);
    }

    public static string BuildMailtoUrl(string email, string? subject = null, string? body = null)
    {
        var url = $"mailto:{email}";
        var query = new List<string>();
        if (!string.IsNullOrWhiteSpace(subject))
            query.Add($"subject={Uri.EscapeDataString(subject)}");
        if (!string.IsNullOrWhiteSpace(body))
            query.Add($"body={Uri.EscapeDataString(body)}");

        return query.Count > 0 ? $"{url}?{string.Join("&", query)}" : url;
    }

    private static string BuildWebClientUrl(string template, string email, string encodedEmail, string? subject, string? body)
    {
        var url = template.Replace("{email}", encodedEmail, StringComparison.OrdinalIgnoreCase);
        if (!string.IsNullOrWhiteSpace(subject))
            url += url.Contains('?') ? "&" : "?";
        if (!string.IsNullOrWhiteSpace(subject))
            url += $"subject={Uri.EscapeDataString(subject)}";
        if (!string.IsNullOrWhiteSpace(body))
            url += $"{(url.Contains('?') ? "&" : "?")}body={Uri.EscapeDataString(body)}";
        return url;
    }

    private static void OpenUrl(string url)
    {
        try
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                Process.Start(new ProcessStartInfo(url) { UseShellExecute = true });
            }
            else if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            {
                Process.Start("xdg-open", url);
            }
            else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            {
                Process.Start("open", url);
            }
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Error al intentar abrir la URL: {url}. Exception: {ex.Message}");
        }
    }
}
