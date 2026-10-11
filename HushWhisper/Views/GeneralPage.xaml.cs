using HushWhisper.ViewModels;
using Microsoft.UI.Xaml.Controls;

namespace HushWhisper.Views;

public sealed partial class GeneralPage : Page
{
    public GeneralPage()
    {
        ViewModel = App.GetService<GeneralViewModel>();
        InitializeComponent();
    }

    public GeneralViewModel ViewModel { get; }
}
