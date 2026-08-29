using Avalonia.Controls;

namespace ElectroObraApp.Controls;

public partial class AppShell : UserControl
{
    private const double CompactBreakpoint = 640;

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

        if (width < CompactBreakpoint)
        {
            NavigationSplitView.DisplayMode = SplitViewDisplayMode.Overlay;
            NavigationSplitView.OpenPaneLength = 240;
        }
        else
        {
            NavigationSplitView.DisplayMode = SplitViewDisplayMode.CompactInline;
            NavigationSplitView.OpenPaneLength = 240;
        }
    }
}
