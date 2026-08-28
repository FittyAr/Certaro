using Avalonia.Controls;

namespace ElectroObraApp.Views;

public partial class MainView : UserControl
{
    private const double CompactBreakpoint = 640;

    public MainView()
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
            NavigationSplitView.IsPaneOpen = false;
            NavigationSplitView.OpenPaneLength = 200;
        }
        else
        {
            NavigationSplitView.DisplayMode = SplitViewDisplayMode.CompactInline;
            NavigationSplitView.IsPaneOpen = true;
            NavigationSplitView.OpenPaneLength = 220;
        }
    }
}
