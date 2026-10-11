using HushWhisper.ViewModels;
using Microsoft.UI.Xaml.Controls;

namespace HushWhisper.Views;

public sealed partial class SpeechRecognitionPage : Page
{
    public SpeechRecognitionPage()
    {
        ViewModel = App.GetService<SpeechRecognitionViewModel>();
        InitializeComponent();
    }

    public SpeechRecognitionViewModel ViewModel { get; }
}
