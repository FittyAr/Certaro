using Avalonia.Controls;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.ViewModels;

namespace ElectroObraApp.Views;

public partial class ObrasView : UserControl
{
    public ObrasView()
    {
        InitializeComponent();
    }

    private void OnDataGridDoubleTapped(object? sender, Avalonia.Input.TappedEventArgs e)
    {
        if (DataContext is ObrasViewModel vm && sender is DataGrid dg && dg.SelectedItem is ObraDto dto)
        {
            vm.EditCommand.Execute(dto);
        }
    }
}
