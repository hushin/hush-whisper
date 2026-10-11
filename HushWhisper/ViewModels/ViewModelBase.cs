using CommunityToolkit.Mvvm.ComponentModel;
using HushWhisper.Services;
using Microsoft.UI.Dispatching;

namespace HushWhisper.ViewModels;

public abstract partial class ViewModelBase : ObservableObject
{
    protected DispatcherQueue? Dispatcher { get; } = DispatcherQueue.GetForCurrentThread();

    protected void OnUi(Action action)
    {
        if (Dispatcher is null || Dispatcher.HasThreadAccess)
        {
            SafeInvoke(action);
        }
        else
        {
            Dispatcher.TryEnqueue(() => SafeInvoke(action));
        }
    }

    private static void SafeInvoke(Action action)
    {
        // An exception escaping a DispatcherQueue callback becomes an unhandled
        // WinRT stowed exception (0xc000027b) that crashes the process, so never
        // let one escape.
        try
        {
            action();
        }
        catch (Exception ex)
        {
            AppLog.Write("UI update failed", ex);
        }
    }
}
