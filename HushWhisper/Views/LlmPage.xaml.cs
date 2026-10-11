using HushWhisper.ViewModels;
using Microsoft.UI.Xaml.Controls;

namespace HushWhisper.Views;

public sealed partial class LlmPage : Page
{
    public LlmPage()
    {
        ViewModel = App.GetService<LlmViewModel>();
        InitializeComponent();
    }

    public LlmViewModel ViewModel { get; }
}
