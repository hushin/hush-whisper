using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HushWhisper.Models;
using HushWhisper.Services;

namespace HushWhisper.ViewModels;

public sealed partial class HistoryViewModel : ViewModelBase
{
    private const string AllDates = "すべて";

    private readonly ILogService _log;

    public HistoryViewModel(ILogService log, IRecordingCoordinator coordinator)
    {
        _log = log;

        _dates.Add(AllDates);
        foreach (var date in log.GetAvailableDates())
        {
            _dates.Add(date.ToString("yyyy-MM-dd"));
        }

        SelectedDate = AllDates; // triggers LoadEntries()
        Status = string.Empty;

        coordinator.TranscriptionCompleted += (_, _) => OnUi(Refresh);
    }

    private readonly ObservableCollection<string> _dates = [];

    public ObservableCollection<string> Dates => _dates;

    public ObservableCollection<LogEntry> Entries { get; } = [];

    [ObservableProperty]
    public partial string SelectedDate { get; set; }

    [ObservableProperty]
    public partial string Status { get; set; }

    [RelayCommand]
    private void Refresh()
    {
        _dates.Clear();
        _dates.Add(AllDates);
        foreach (var date in _log.GetAvailableDates())
        {
            _dates.Add(date.ToString("yyyy-MM-dd"));
        }

        LoadEntries();
    }

    [RelayCommand]
    private void Delete(LogEntry? entry)
    {
        if (entry is null)
        {
            return;
        }

        _log.Delete(entry.Id);
        Entries.Remove(entry);
        Status = "削除しました";
    }

    [RelayCommand]
    private void DeleteAll()
    {
        var count = _log.DeleteAll();
        Entries.Clear();
        Refresh();
        Status = $"{count} 件削除しました";
    }

    [RelayCommand]
    private void Copy(LogEntry? entry)
    {
        if (entry is null)
        {
            return;
        }

        var text = entry.RefinedText ?? entry.RawText;
        var dataPackage = new Windows.ApplicationModel.DataTransfer.DataPackage();
        dataPackage.SetText(text);
        Windows.ApplicationModel.DataTransfer.Clipboard.SetContent(dataPackage);
        Status = "コピーしました";
    }

    partial void OnSelectedDateChanged(string value) => LoadEntries();

    private void LoadEntries()
    {
        Entries.Clear();

        IEnumerable<LogEntry> entries = SelectedDate == AllDates
            ? _log.GetRecent(100)
            : _log.GetForDate(DateOnly.Parse(SelectedDate));

        foreach (var entry in entries)
        {
            Entries.Add(entry);
        }
    }
}
