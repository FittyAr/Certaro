using Avalonia.Controls;
using Avalonia.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.ViewModels;

namespace ElectroObraApp.Views;

public partial class TiposMovimientoView : UserControl
{
    public TiposMovimientoView()
    {
        InitializeComponent();
    }

    private void OnDataGridDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (DataContext is TiposMovimientoViewModel vm && sender is DataGrid dg && dg.SelectedItem is TipoMovimientoDto dto)
            vm.EditCommand.Execute(dto);
    }
}
