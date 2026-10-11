using HushWhisper.Models;
using HushWhisper.ViewModels;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace HushWhisper.Views;

public sealed partial class HistoryPage : Page
{
    public HistoryPage()
    {
        ViewModel = App.GetService<HistoryViewModel>();
        InitializeComponent();
    }

    public HistoryViewModel ViewModel { get; }

    private void Copy_Click(object sender, RoutedEventArgs e)
    {
        if ((sender as FrameworkElement)?.DataContext is LogEntry entry)
        {
            ViewModel.CopyCommand.Execute(entry);
        }
    }

    private void Delete_Click(object sender, RoutedEventArgs e)
    {
        if ((sender as FrameworkElement)?.DataContext is LogEntry entry)
        {
            ViewModel.DeleteCommand.Execute(entry);
        }
    }
}
