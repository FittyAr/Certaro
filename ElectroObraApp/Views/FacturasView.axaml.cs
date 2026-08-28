using Avalonia.Controls;
using Avalonia.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.ViewModels;

namespace ElectroObraApp.Views;

public partial class FacturasView : UserControl
{
    public FacturasView()
    {
        InitializeComponent();
    }

    private void OnDataGridDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (DataContext is FacturasViewModel vm && sender is DataGrid grid && grid.SelectedItem is FacturaDto dto)
        {
            vm.EditCommand.Execute(dto);
        }
    }
}
