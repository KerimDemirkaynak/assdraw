#include "assdraw.hpp"
#include "theme.hpp"

#include <array>
#include <wx/aui/dockart.h>
#include <wx/settings.h>

namespace {
using ViewField = std::pair<const char*, wxColour*>;
std::array<ViewField, 10> ViewFields(ASSDrawFrame& frame)
{
    return {{{"canvas_bg", &frame.colors.canvas_bg},
        {"guidelines", &frame.colors.canvas_shape_guideline},
        {"main_points", &frame.colors.canvas_shape_mainpoint},
        {"control_points", &frame.colors.canvas_shape_controlpoint},
        {"selected_points", &frame.colors.canvas_shape_selectpoint},
        {"library_bg", &frame.colors.library_libarea},
        {"library_shapes", &frame.colors.library_shape},
        {"origin", &frame.colors.origin},
        {"ruler_h", &frame.colors.ruler_h}, {"ruler_v", &frame.colors.ruler_v}}};
}
}

void ASSDrawFrame::SaveThemeViewColors(bool dark)
{
    const wxString prefix = dark ? "/ui/theme_colors/dark/" : "/ui/theme_colors/light/";
    for (const auto& field : ViewFields(*this))
        config->Write(prefix + field.first, field.second->GetAsString(wxC2S_CSS_SYNTAX));
}

void ASSDrawFrame::LoadThemeViewColors(bool dark)
{
    if (dark) {
        colors.canvas_bg = AssdrawTheme::Background();
        colors.canvas_shape_guideline = wxColour(150, 154, 160);
        colors.canvas_shape_mainpoint = wxColour(255, 112, 112);
        colors.canvas_shape_controlpoint = wxColour(94, 220, 138);
        colors.canvas_shape_selectpoint = wxColour(104, 190, 255);
        colors.library_libarea = AssdrawTheme::Surface();
        colors.library_shape = wxColour(100, 181, 246);
        colors.origin = wxColour(255, 112, 112);
        colors.ruler_h = wxColour(181, 205, 240);
        colors.ruler_v = wxColour(240, 178, 190);
    }
    const wxString prefix = dark ? "/ui/theme_colors/dark/" : "/ui/theme_colors/light/";
    for (const auto& field : ViewFields(*this)) {
        wxString value;
        if (config->Read(prefix + field.first, &value)) {
            const wxColour colour(value);
            if (colour.IsOk()) *field.second = colour;
        }
    }
}

void ASSDrawFrame::OnChooseTheme(wxCommandEvent& event)
{
    static const char* modes[] = {"system", "light", "dark"};
    const int index = event.GetId() - 6100;
    if (index >= 0 && index < 3) {
        const wxString mode = modes[index];
        CallAfter([this, mode] {
            if (!ApplyTheme(mode))
                wxMessageBox(_("The selected theme could not be applied."), _("Theme"), wxOK | wxICON_ERROR, this);
        });
    }
}

bool ASSDrawFrame::ApplyTheme(const wxString& mode)
{
    if (!AssdrawTheme::SetAppearance(mode))
        return false;
    RefreshTheme();
    config->Write("/ui/theme", mode);
    SaveSettings();
    if (!AssdrawI18n::SaveConfig(*config, configfile)) {
        wxMessageBox(_("Could not save the theme preference."), _("Theme"), wxOK | wxICON_ERROR, this);
        return false;
    }
    return true;
}

void ASSDrawFrame::OnSystemColoursChanged(wxSysColourChangedEvent& event)
{
    event.Skip();
    if (!theme_ready || theme_refresh_pending)
        return;
    theme_refresh_pending = true;
    CallAfter([this] {
        theme_refresh_pending = false;
        if (!IsBeingDeleted()) RefreshTheme();
    });
}

void ASSDrawFrame::RefreshTheme()
{
    if (!theme_ready)
        return;
    AssdrawTheme::PreparePalette();
    const bool dark = AssdrawTheme::IsDark();
    const bool changed = dark != view_is_dark;
    if (changed) {
        SaveThemeViewColors(view_is_dark);
        LoadThemeViewColors(dark);
        view_is_dark = dark;
        settingsdlg->RefreshThemeViewColors();
    }
    Freeze();
    AssdrawTheme::ApplyControls(this);
    copy_button->SetBitmap(ASSDRAW_ICON(Copy));
    auto* art = m_mgr.GetArtProvider();
    const wxColour surface = AssdrawTheme::Surface(), text = AssdrawTheme::Text();
    art->SetColour(wxAUI_DOCKART_BACKGROUND_COLOUR, surface);
    art->SetColour(wxAUI_DOCKART_SASH_COLOUR, surface);
    art->SetColour(wxAUI_DOCKART_INACTIVE_CAPTION_COLOUR, surface);
    art->SetColour(wxAUI_DOCKART_INACTIVE_CAPTION_GRADIENT_COLOUR, surface);
    art->SetColour(wxAUI_DOCKART_INACTIVE_CAPTION_TEXT_COLOUR, text);
    art->SetColour(wxAUI_DOCKART_ACTIVE_CAPTION_COLOUR, wxSystemSettings::GetColour(wxSYS_COLOUR_HIGHLIGHT));
    art->SetColour(wxAUI_DOCKART_ACTIVE_CAPTION_GRADIENT_COLOUR, wxSystemSettings::GetColour(wxSYS_COLOUR_HIGHLIGHT));
    art->SetColour(wxAUI_DOCKART_ACTIVE_CAPTION_TEXT_COLOUR, wxSystemSettings::GetColour(wxSYS_COLOUR_HIGHLIGHTTEXT));
    art->SetColour(wxAUI_DOCKART_BORDER_COLOUR, wxSystemSettings::GetColour(wxSYS_COLOUR_3DSHADOW));
    const int selection = AssdrawTheme::Requested() == "dark" ? 6102 :
        AssdrawTheme::Requested() == "light" ? 6101 : 6100;
    themeMenu->Check(selection, true);
    ApplyViewColours();
    if (changed)
        m_canvas->PrepareBackgroundBitmap(-1.0);
    for (auto* preview : shapelib->GetShapePreviews())
        preview->UpdateTheme();
    m_mgr.Update();
    Thaw();
    Refresh();
}
