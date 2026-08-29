using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.ViewModels;

public sealed class CommandPaletteItem
{
    public required string Title { get; init; }
    public required string Route { get; init; }
    public required string Category { get; init; }
    public string Keywords { get; init; } = string.Empty;
}

public partial class CommandPaletteViewModel : ViewModelBase
{
    private readonly INavigationService _navigationService;
    private readonly ILocalizationService _localizationService;
    private readonly IReadOnlyList<CommandPaletteItem> _allCommands;

    public event Action? CloseRequested;

    [ObservableProperty]
    private string _searchQuery = string.Empty;

    [ObservableProperty]
    private CommandPaletteItem? _selectedItem;

    public ObservableCollection<CommandPaletteItem> FilteredCommands { get; } = new();

    public CommandPaletteViewModel(
        INavigationService navigationService,
        ILocalizationService localizationService)
    {
        _navigationService = navigationService;
        _localizationService = localizationService;
        _allCommands = BuildCommands();
        Reset();
    }

    public void Reset()
    {
        SearchQuery = string.Empty;
        FilterCommands();
        SelectedItem = FilteredCommands.FirstOrDefault();
    }

    partial void OnSearchQueryChanged(string value) => FilterCommands();

    [RelayCommand]
    private void ExecuteSelected()
    {
        if (SelectedItem is null)
        {
            return;
        }

        _navigationService.NavigateTo(SelectedItem.Route);
        CloseRequested?.Invoke();
    }

    [RelayCommand]
    private void ExecuteItem(CommandPaletteItem? item)
    {
        if (item is null)
        {
            return;
        }

        SelectedItem = item;
        ExecuteSelectedCommand.Execute(null);
    }

    private void FilterCommands()
    {
        var query = SearchQuery.Trim();
        FilteredCommands.Clear();

        var matches = string.IsNullOrWhiteSpace(query)
            ? _allCommands
            : _allCommands.Where(c =>
                c.Title.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                c.Category.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                c.Keywords.Contains(query, StringComparison.OrdinalIgnoreCase));

        foreach (var item in matches)
        {
            FilteredCommands.Add(item);
        }

        SelectedItem = FilteredCommands.FirstOrDefault();
    }

    private IReadOnlyList<CommandPaletteItem> BuildCommands() =>
    [
        CreateItem("dashboard", "Menu.Dashboard", "Menu.Group.Operacion"),
        CreateItem("movimientos", "Menu.Movimientos", "Menu.Group.Operacion"),
        CreateItem("clientes", "Menu.Clientes", "Menu.Group.Comercial"),
        CreateItem("obras", "Menu.Obras", "Menu.Group.Comercial"),
        CreateItem("certificados", "Menu.Certificados", "Menu.Group.Comercial"),
        CreateItem("facturas", "Menu.Facturas", "Menu.Group.Comercial"),
        CreateItem("empleados", "Menu.Empleados", "Menu.Group.Personal"),
        CreateItem("asistencia", "Menu.Asistencia", "Menu.Group.Personal"),
        CreateItem("liquidaciones", "Menu.Liquidaciones", "Menu.Group.Personal"),
        CreateItem("configuracion", "Menu.Settings", "Menu.Group.Sistema")
    ];

    private CommandPaletteItem CreateItem(string route, string titleKey, string categoryKey) =>
        new()
        {
            Route = route,
            Title = _localizationService.GetString(titleKey),
            Category = _localizationService.GetString(categoryKey),
            Keywords = route
        };
}
