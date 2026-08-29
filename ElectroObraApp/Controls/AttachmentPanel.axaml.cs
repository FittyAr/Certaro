using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Microsoft.Extensions.DependencyInjection;
using ElectroObraApp.ViewModels;

namespace ElectroObraApp.Controls;

public partial class AttachmentPanel : UserControl
{
    public static readonly StyledProperty<string> EntidadTipoProperty =
        AvaloniaProperty.Register<AttachmentPanel, string>(nameof(EntidadTipo), string.Empty);

    public static readonly StyledProperty<Guid> EntidadIdProperty =
        AvaloniaProperty.Register<AttachmentPanel, Guid>(nameof(EntidadId));

    private AttachmentPanelViewModel? _viewModel;
    private bool _isInitialized;

    public string EntidadTipo
    {
        get => GetValue(EntidadTipoProperty);
        set => SetValue(EntidadTipoProperty, value);
    }

    public Guid EntidadId
    {
        get => GetValue(EntidadIdProperty);
        set => SetValue(EntidadIdProperty, value);
    }

    public AttachmentPanel()
    {
        InitializeComponent();
        Loaded += OnLoaded;
    }

    private async void OnLoaded(object? sender, RoutedEventArgs e)
    {
        if (_isInitialized)
            return;

        if (Avalonia.Application.Current is App app && app.Services != null)
        {
            _viewModel = app.Services.GetRequiredService<AttachmentPanelViewModel>();
            DataContext = _viewModel;
        }

        DragDrop.SetAllowDrop(DropZone, true);
        DropZone.AddHandler(DragDrop.DragOverEvent, OnDragOver);
        DropZone.AddHandler(DragDrop.DropEvent, OnDrop);

        _isInitialized = true;
        await SyncContextAsync();
    }

    protected override async void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);

        if (!_isInitialized)
            return;

        if (change.Property == EntidadTipoProperty || change.Property == EntidadIdProperty)
            await SyncContextAsync();
    }

    private async Task SyncContextAsync()
    {
        if (_viewModel == null)
            return;

        _viewModel.EntidadTipo = EntidadTipo;
        _viewModel.EntidadId = EntidadId;
        await _viewModel.LoadAsync();
    }

    private void OnDragOver(object? sender, DragEventArgs e)
    {
        e.DragEffects = e.DataTransfer.Formats.Contains(DataFormat.File)
            ? DragDropEffects.Copy
            : DragDropEffects.None;
    }

    private async void OnDrop(object? sender, DragEventArgs e)
    {
        if (_viewModel == null || !_viewModel.CanManage || !e.DataTransfer.Formats.Contains(DataFormat.File))
            return;

        var files = e.DataTransfer.TryGetFiles();
        if (files == null)
            return;

        var paths = files
            .OfType<IStorageFile>()
            .Select(f => f.TryGetLocalPath())
            .Where(path => !string.IsNullOrWhiteSpace(path))
            .Cast<string>()
            .ToArray();

        if (paths.Length > 0 && _viewModel.AddFilesCommand.CanExecute(paths))
            await _viewModel.AddFilesCommand.ExecuteAsync(paths);
    }

    private async void BrowseFiles_Click(object? sender, RoutedEventArgs e)
    {
        if (_viewModel == null || !_viewModel.CanManage)
            return;

        var topLevel = TopLevel.GetTopLevel(this);
        if (topLevel?.StorageProvider == null)
            return;

        var files = await topLevel.StorageProvider.OpenFilePickerAsync(new FilePickerOpenOptions
        {
            AllowMultiple = true
        });

        var paths = files
            .Select(f => f.TryGetLocalPath())
            .Where(path => !string.IsNullOrWhiteSpace(path))
            .Cast<string>()
            .ToArray();

        if (paths.Length > 0 && _viewModel.AddFilesCommand.CanExecute(paths))
            await _viewModel.AddFilesCommand.ExecuteAsync(paths);
    }
}
