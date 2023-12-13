#include <QGuiApplication>
#include <QDebug>
#include <QWindow>

#include <qpa/qplatformwindow_p.h>

namespace rust::servo {

using c_void = void;

// https://blog.david-redondo.de/qt/kde/2022/12/09/wayland-native-interface.html

// QPlatformNativeInterface *nativeInterface = QGuiApplication::platformNativeInterface();
// if (!nativeInterface) {
//     return nullptr;
// }

// auto display = static_cast<wl_display*>(nativeInterface->nativeResourceForIntegration("wl_display"));

void* waylandDisplayHandle()
{
    if (auto waylandApp = qGuiApp->nativeInterface<QNativeInterface::QWaylandApplication>())
    {
        qWarning() << Q_FUNC_INFO << "found wayland display!";
        return waylandApp->display();
    }
    else
    {
        qWarning() << Q_FUNC_INFO << "no display!";
        return nullptr;
    }
}

// void* x11DisplayHandle()
// {
//     if (auto x11App = qGuiApp->nativeInterface<QNativeInterface::QX11Application>())
//     {
//         qWarning() << Q_FUNC_INFO << "found x11 display!";
//         return x11App->connection();
//     }
//     else
//     {
//         qWarning() << Q_FUNC_INFO << "no display!";
//         return nullptr;
//     }
// }

void* waylandWindowHandle()
{
    // TODO: instead pass in a QWindow from a QML attached property
    //
    // QWindow* window = QGuiApplication::focusWindow();
    auto windows = QGuiApplication::allWindows();
    if (windows.isEmpty())
    {
        qWarning() << Q_FUNC_INFO << "no windows!";
        return nullptr;
    }

    QWindow* window = windows.first();
    if (window == nullptr)
    {
        qWarning() << Q_FUNC_INFO << "no first window!";
        return nullptr;
    }

    if (auto waylandWindow = window->nativeInterface<QNativeInterface::Private::QWaylandWindow>())
    {
        qWarning() << Q_FUNC_INFO << "found wayland window!";
        return waylandWindow->surface();
    }
    else
    {
        qWarning() << Q_FUNC_INFO << "no window!";
        return nullptr;
    }
}

// void* windowHandle(QWindow* window)
// {
//     // QPlatformNativeInterface *nativeInterface = QGuiApplication::platformNativeInterface();
//     // if (!nativeInterface) {
//     //     return nullptr;
//     // }
//     // auto surface = static_cast<wl_surface*>(nativeInterface->nativeResourceForWindow("surface", window));

//     if (auto waylandWindow = window->nativeInterface<QNativeInterface::Private::QWaylandWindow>())
//     {
//         return waylandWindow->wlSurface();
//     }
// }

} // namespace rust::servo
