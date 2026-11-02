#include "theme.hpp"
#include "localization.hpp"
#include "icons.hpp"

#include <wx/app.h>
#include <wx/fileconf.h>
#include <wx/filename.h>
#include <wx/frame.h>
#include <wx/dialog.h>
#include <wx/panel.h>
#include <wx/statusbr.h>
#include <wx/html/htmlwin.h>
#include <wx/propgrid/propgrid.h>
#include <wx/settings.h>
#include <wx/textctrl.h>
#include <wx/toolbar.h>
#include <wx/wfstream.h>

#ifdef __WXGTK3__
#include <gtk/gtk.h>
#endif

namespace {
wxString requested = "system";
#ifdef __WXGTK3__
wxString original_gtk_theme;

wxString GtkThemeName()
{
    gchar* name = nullptr;
    g_object_get(gtk_settings_get_default(), "gtk-theme-name", &name, nullptr);
    const wxString theme = wxString::FromUTF8(name ? name : "Adwaita");
    g_free(name);
    return theme;
}

void FlushGtkPalette()
{
    gtk_style_context_reset_widgets(gdk_screen_get_default());
    g_object_notify(G_OBJECT(gtk_settings_get_default()), "gtk-theme-name");
}

wxString DarkGtkTheme(const wxString& theme)
{
    // Some desktop themes ship a separate name, e.g. Yaru-dark, instead of
    // a gtk-dark.css variant selected by gtk-application-prefer-dark-theme.
    wxArrayString roots;
    roots.Add(wxString::FromUTF8(g_get_user_data_dir()) + "/themes/");
    roots.Add(wxString::FromUTF8(g_get_home_dir()) + "/.themes/");
    for (const gchar* const* path = g_get_system_data_dirs(); *path; ++path)
        roots.Add(wxString::FromUTF8(*path) + "/themes/");
    for (const char* suffix : {"-dark", "-Dark"}) {
        const wxString candidate = theme + suffix;
        for (const auto& root : roots)
            if (wxFileExists(root + candidate + "/gtk-3.0/gtk.css"))
                return candidate;
    }
    // Adwaita has a built-in dark variant even on minimal/headless systems.
    return "Adwaita";
}
#endif
}

namespace AssdrawTheme {
wxString ReadPreference()
{
    const wxString path = AssdrawI18n::ConfigPath();
    if (!wxFileExists(path))
        return "system";
    wxFileInputStream stream(path);
    wxFileConfig config(stream);
    const wxString mode = config.Read("/ui/theme", "system");
    return mode == "light" || mode == "dark" ? mode : wxString("system");
}

const wxString& Requested() { return requested; }

bool SetAppearance(const wxString& mode)
{
    wxApp::Appearance appearance;
    if (mode == "light") appearance = wxApp::Appearance::Light;
    else if (mode == "dark") appearance = wxApp::Appearance::Dark;
    else if (mode == "system") appearance = wxApp::Appearance::System;
    else return false;
    if (wxTheApp->SetAppearance(appearance) != wxApp::AppearanceResult::Ok)
        return false;
#ifdef __WXGTK3__
    FlushGtkPalette();
#endif
    requested = mode;
    PreparePalette();
    return true;
}

void PreparePalette()
{
#ifdef __WXGTK3__
    if (g_getenv("GTK_THEME")) return;
    gboolean prefer_dark = FALSE;
    g_object_get(gtk_settings_get_default(), "gtk-application-prefer-dark-theme", &prefer_dark, nullptr);
    if (!prefer_dark && !original_gtk_theme.empty()) {
        const wxString theme = original_gtk_theme;
        original_gtk_theme.clear();
        g_object_set(gtk_settings_get_default(), "gtk-theme-name", theme.utf8_str().data(), nullptr);
        FlushGtkPalette();
    } else if (prefer_dark && !IsDark() && original_gtk_theme.empty()) {
        original_gtk_theme = GtkThemeName();
        const wxString theme = DarkGtkTheme(original_gtk_theme);
        g_object_set(gtk_settings_get_default(), "gtk-theme-name", theme.utf8_str().data(), nullptr);
        FlushGtkPalette();
    }
#endif
}

bool IsDark() { return wxSystemSettings::GetAppearance().IsDark(); }
wxColour Background() { return wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW); }
wxColour Surface() { return wxSystemSettings::GetColour(wxSYS_COLOUR_3DFACE); }
wxColour Text() { return wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT); }


void ApplyControls(wxWindow* window)
{
    if (!window || window->IsBeingDeleted())
        return;
    const bool input = wxDynamicCast(window, wxTextCtrl) != nullptr;
    // Standard buttons, checkboxes, sliders and native dialogs use wxWidgets'
    // own dark-mode rendering. Colour only our containers and editors here.
    if (input || wxDynamicCast(window, wxPanel) || wxDynamicCast(window, wxFrame)
        || wxDynamicCast(window, wxDialog) || wxDynamicCast(window, wxStatusBar)) {
        window->SetBackgroundColour(input ? Background() : Surface());
        window->SetForegroundColour(Text());
    }
    if (auto* grid = wxDynamicCast(window, wxPropertyGrid)) {
        grid->SetCellBackgroundColour(Background());
        grid->SetCellTextColour(Text());
        grid->SetEmptySpaceColour(Surface());
        grid->SetMarginColour(Surface());
        grid->SetCaptionBackgroundColour(Surface());
        grid->SetCaptionTextColour(Text());
        grid->SetLineColour(wxSystemSettings::GetColour(wxSYS_COLOUR_3DSHADOW));
        grid->SetSelectionBackgroundColour(wxSystemSettings::GetColour(wxSYS_COLOUR_HIGHLIGHT));
        grid->SetSelectionTextColour(wxSystemSettings::GetColour(wxSYS_COLOUR_HIGHLIGHTTEXT));
    }
    if (auto* bar = wxDynamicCast(window, wxToolBar))
        AssdrawIcons::RefreshToolbar(bar);
    if (auto* html = wxDynamicCast(window, wxHtmlWindow)) {
        html->SetBackgroundColour(Background());
        html->SetPage(AssdrawI18n::AboutHtml());
    }
    for (wxWindow* child : window->GetChildren())
        ApplyControls(child);
    window->Refresh();
}
}
