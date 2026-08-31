using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class AppShell : UserControl
{
    private const double OverlayBreakpoint = 768;

    public AppShell()
    {
        InitializeComponent();
        UpdateNavigationLayout(Bounds.Width);
    }

    protected override void OnSizeChanged(SizeChangedEventArgs e)
    {
        base.OnSizeChanged(e);
        UpdateNavigationLayout(e.NewSize.Width);
    }

    private void UpdateNavigationLayout(double width)
    {
        if (NavigationSplitView is null)
        {
            return;
        }

        NavigationSplitView.OpenPaneLength = 260;

        if (width < OverlayBreakpoint)
        {
            NavigationSplitView.DisplayMode = SplitViewDisplayMode.Overlay;
        }
        else
        {
            NavigationSplitView.DisplayMode = SplitViewDisplayMode.CompactInline;
        }
    }
}
