using Avalonia.Controls;
using Avalonia.Input;
using ElectroObraApp.Application.DTOs;
using ElectroObraApp.ViewModels;

namespace ElectroObraApp.Views;

public partial class CategoriasView : UserControl
{
    public CategoriasView()
    {
        InitializeComponent();
    }

    private void OnDataGridDoubleTapped(object? sender, TappedEventArgs e)
    {
        if (DataContext is CategoriasViewModel vm && sender is DataGrid dg && dg.SelectedItem is CategoriaDto dto)
            vm.EditCommand.Execute(dto);
    }
}
