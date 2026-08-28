using System;
using System.Collections.Generic;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using ElectroObraApp.Application.Interfaces;

namespace ElectroObraApp.Services;

public class FileSaveDialogService : IFileSaveDialogService
{
    public async Task<bool> SaveFileAsync(byte[] content, string suggestedFileName, string extension)
    {
        if (global::Avalonia.Application.Current?.ApplicationLifetime is not IClassicDesktopStyleApplicationLifetime desktop ||
            desktop.MainWindow is null)
        {
            return false;
        }

        var tcs = new TaskCompletionSource<bool>();

        await Dispatcher.UIThread.InvokeAsync(async () =>
        {
            try
            {
                var storageProvider = desktop.MainWindow.StorageProvider;
                var fileTypes = new FilePickerFileType(extension.ToUpperInvariant())
                {
                    Patterns = new[] { $"*.{extension}" }
                };

                var file = await storageProvider.SaveFilePickerAsync(new FilePickerSaveOptions
                {
                    SuggestedFileName = suggestedFileName,
                    FileTypeChoices = new List<FilePickerFileType> { fileTypes }
                });

                if (file is null)
                {
                    tcs.TrySetResult(false);
                    return;
                }

                await using var stream = await file.OpenWriteAsync();
                await stream.WriteAsync(content);
                tcs.TrySetResult(true);
            }
            catch (Exception)
            {
                tcs.TrySetResult(false);
            }
        });

        return await tcs.Task;
    }
}
