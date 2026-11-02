#include "theme.hpp"
#include "localization.hpp"

#include <wx/app.h>
#include <wx/fileconf.h>
#include <wx/filename.h>
#include <wx/frame.h>
#include <wx/dialog.h>
#include <wx/panel.h>
#include <wx/statusbr.h>
#include <wx/html/htmlwin.h>
#include <wx/image.h>
#include <wx/propgrid/propgrid.h>
#include <wx/settings.h>
#include <wx/textctrl.h>
#include <wx/toolbar.h>
#include <wx/wfstream.h>

namespace {
wxString requested = "system";

wxBitmap InvertGreys(const wxBitmap& bitmap)
{
    if (!bitmap.IsOk())
        return bitmap;
    wxImage image = bitmap.ConvertToImage();
    if (!image.HasAlpha()) image.InitAlpha();
    unsigned char* pixels = image.GetData();
    for (int i = 0; i < image.GetWidth() * image.GetHeight(); ++i) {
        unsigned char* p = pixels + i * 3;
        if (p[0] == p[1] && p[1] == p[2])
            p[0] = p[1] = p[2] = 255 - p[0];
    }
    wxBitmap result(image);
    result.SetScaleFactor(bitmap.GetScaleFactor());
    return result;
}
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
    requested = mode;
    return true;
}

bool IsDark() { return wxSystemSettings::GetAppearance().IsDark(); }
wxColour Background() { return wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOW); }
wxColour Surface() { return wxSystemSettings::GetColour(wxSYS_COLOUR_3DFACE); }
wxColour Text() { return wxSystemSettings::GetColour(wxSYS_COLOUR_WINDOWTEXT); }

wxBitmapBundle ToolBitmap(const wxBitmap& bitmap)
{
    wxImage image = bitmap.ConvertToImage();
    // Windows BMP resources use this matte; XPM resources already have alpha.
    if (!image.HasAlpha() && !image.HasMask()) {
        const wxColour corner(image.GetRed(0, 0), image.GetGreen(0, 0), image.GetBlue(0, 0));
        if (corner == wxColour(192, 192, 192))
            image.SetMaskColour(192, 192, 192);
    }
    if (!image.HasAlpha()) image.InitAlpha();
    wxBitmap result(image);
    result.SetScaleFactor(bitmap.GetScaleFactor());
    return wxBitmapBundle::FromBitmap(IsDark() ? InvertGreys(result) : result);
}

void ApplyControls(wxWindow* window, bool invert_tool_icons)
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
    if (auto* bar = wxDynamicCast(window, wxToolBar)) {
        if (invert_tool_icons) {
            for (size_t i = 0; i < bar->GetToolsCount(); ++i) {
                auto* tool = bar->GetToolByPos(i);
                if (tool->IsButton()) {
                    const wxBitmap bitmap = tool->GetNormalBitmapBundle().GetBitmapFor(bar);
                    bar->SetToolNormalBitmap(tool->GetId(), wxBitmapBundle::FromBitmap(InvertGreys(bitmap)));
                }
            }
        }
        bar->Realize();
    }
    if (auto* html = wxDynamicCast(window, wxHtmlWindow)) {
        html->SetBackgroundColour(Background());
        html->SetPage(AssdrawI18n::AboutHtml());
    }
    for (wxWindow* child : window->GetChildren())
        ApplyControls(child, invert_tool_icons);
    window->Refresh();
}
}
