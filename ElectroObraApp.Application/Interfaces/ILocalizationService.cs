namespace ElectroObraApp.Application.Interfaces;

public interface ILocalizationService
{
    string CurrentLanguage { get; }
    string GetString(string key);
    void SetLanguage(string languageCode);
    event EventHandler? LanguageChanged;
}
