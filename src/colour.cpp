#include "colour.hpp"

#include <array>
#include <wx/odcombo.h>
#include <wx/propgrid/editors.h>
#include <wx/propgrid/propgrid.h>

#ifdef __WXMSW__
#include <wx/msw/wrapwin.h>
#include <colordlg.h>
#include <dlgs.h>
#endif

namespace {
// The palette order in pinned wxWidgets 3.3.4.
// Numeric choice values stay unchanged: they identify RGB colours, not labels.
constexpr std::array<const char*, 18> colour_names = {{
    wxTRANSLATE("Black"), wxTRANSLATE("Maroon"), wxTRANSLATE("Navy"),
    wxTRANSLATE("Purple"), wxTRANSLATE("Teal"), wxTRANSLATE("Gray"),
    wxTRANSLATE("Green"), wxTRANSLATE("Olive"), wxTRANSLATE("Brown"),
    wxTRANSLATE("Blue"), wxTRANSLATE("Fuchsia"), wxTRANSLATE("Red"),
    wxTRANSLATE("Orange"), wxTRANSLATE("Silver"), wxTRANSLATE("Lime"),
    wxTRANSLATE("Aqua"), wxTRANSLATE("Yellow"), wxTRANSLATE("White")
}};

#ifdef __WXMSW__
void SetNativeLabel(HWND dialog, int id, const wxString& label)
{
    if (HWND control = ::GetDlgItem(dialog, id))
        ::SetWindowTextW(control, label.wc_str());
}

void SetPaletteLabel(HWND dialog, int palette_id, const wxString& label)
{
    HWND palette = ::GetDlgItem(dialog, palette_id);
    if (!palette) return;
    RECT palette_rect{};
    ::GetWindowRect(palette, &palette_rect);
    // The native template may give both palette captions IDC_STATIC (-1),
    // so GetDlgItem cannot distinguish them. Find the nearest text above each
    // palette by geometry instead of matching the operating-system language.
    HWND caption = nullptr;
    int nearest = 0;
    for (HWND child = ::GetWindow(dialog, GW_CHILD); child;
         child = ::GetWindow(child, GW_HWNDNEXT)) {
        wchar_t name[32]{};
        ::GetClassNameW(child, name, 32);
        if (wxString(name).CmpNoCase("Static") != 0 || !::GetWindowTextLengthW(child))
            continue;
        RECT rect{};
        ::GetWindowRect(child, &rect);
        if (rect.right > palette_rect.left && rect.left < palette_rect.right
            && rect.bottom <= palette_rect.top && (!caption || rect.bottom > nearest)) {
            caption = child;
            nearest = rect.bottom;
        }
    }
    if (caption) ::SetWindowTextW(caption, label.wc_str());
}
#endif
}

AssdrawColourDialog::AssdrawColourDialog(wxWindow* parent, const wxColourData* data)
    : wxColourDialog(parent, data)
{
    SetTitle(_("Colour"));
}

#ifdef __WXMSW__
void AssdrawColourDialog::MSWOnInitDone(WXHWND native_dialog)
{
    wxColourDialog::MSWOnInitDone(native_dialog);
    const HWND dialog = static_cast<HWND>(native_dialog);
    SetPaletteLabel(dialog, COLOR_BOX1, "&" + _("Basic colours") + ":");
    SetPaletteLabel(dialog, COLOR_CUSTOM1, "&" + _("Custom colours") + ":");
    SetNativeLabel(dialog, IDOK, _("OK"));
    SetNativeLabel(dialog, IDCANCEL, _("Cancel"));
    SetNativeLabel(dialog, pshHelp, _("Help"));
    SetNativeLabel(dialog, COLOR_MIX, "&" + _("Define custom colours") + " >>");
    SetNativeLabel(dialog, COLOR_ADD, "&" + _("Add to custom colours"));
    SetNativeLabel(dialog, COLOR_REDACCEL, "&" + _("Red") + ":");
    SetNativeLabel(dialog, COLOR_GREENACCEL, "&" + _("Green") + ":");
    SetNativeLabel(dialog, COLOR_BLUEACCEL, "&" + _("Blue") + ":");
    SetNativeLabel(dialog, COLOR_HUEACCEL, "&" + _("Hue:"));
    SetNativeLabel(dialog, COLOR_SATACCEL, "&" + _("Sat:"));
    SetNativeLabel(dialog, COLOR_LUMACCEL, "&" + _("Lum:"));
    SetNativeLabel(dialog, COLOR_SOLID_LEFT, _("Colour"));
    SetNativeLabel(dialog, COLOR_SOLID_RIGHT, "|" + _("Solid"));
}
#endif

AssdrawColourProperty::AssdrawColourProperty(const wxString& label, const wxString& name,
                                           const wxColour& value)
    : wxColourProperty(label, name, value)
{
    UpdateTranslations();
}

void AssdrawColourProperty::UpdateTranslations()
{
    // Detach the wxWidgets cache without SetChoices(), which clears the value.
    // Keep pending settings, the selected colour index and Modified flags.
    m_choices = m_choices.Copy();
    for (unsigned i = 0; i < m_choices.GetCount(); ++i) {
        const int value = m_choices.GetValue(i);
        if (value == wxPG_COLOUR_CUSTOM)
            m_choices.Item(i).SetText(_("Custom"));
        else if (value >= 0 && value < static_cast<int>(colour_names.size()))
            m_choices.Item(i).SetText(wxGetTranslation(colour_names[value]));
    }
    if (wxPropertyGrid* grid = GetGrid()) {
        if (grid->GetSelection() == this) {
            if (wxWindow* control = grid->GetEditorControl())
                GetEditorClass()->SetItems(control, m_choices.GetLabels());
            grid->RefreshEditor();
        }
        grid->RefreshProperty(this);
    }
}

bool AssdrawColourProperty::IntToValue(wxVariant& value, int number,
                                      wxPGPropValFormatFlags flags) const
{
    // User selections are deferred to OnEvent, so every chooser uses our
    // localization hook. Programmatic conversions must never open a dialog.
    if (number >= 0 && number == GetCustomColourIndex()
        && !(flags & wxPGPropValFormatFlags::PropertySpecific))
        return false;
    return wxColourProperty::IntToValue(value, number, flags);
}

bool AssdrawColourProperty::OnEvent(wxPropertyGrid* grid, wxWindow* primary, wxEvent& event)
{
    bool choose = grid->IsMainButtonEvent(event);
    if (event.GetEventType() == wxEVT_COMBOBOX) {
        auto* control = wxDynamicCast(grid->GetEditorControl(), wxOwnerDrawnComboBox);
        choose = control && GetCustomColourIndex() >= 0
            && control->GetSelection() == GetCustomColourIndex();
    }
    if (!choose) return wxColourProperty::OnEvent(grid, primary, event);
    if (grid->WasValueChangedInEvent()) return false;

    wxColourData data;
    data.SetChooseFull(true);
    data.SetChooseAlpha(GetAttributeAsLong(wxPG_COLOUR_HAS_ALPHA, 0) != 0);
    data.SetColour(GetVal().m_colour);
    for (int i = 0; i < wxColourData::NUM_CUSTOM; ++i) {
        const unsigned char level = i * (256 / wxColourData::NUM_CUSTOM);
        data.SetCustomColour(i, wxColour(level, level, level));
    }
    AssdrawColourDialog dialog(grid, &data);
    if (dialog.ShowModal() != wxID_OK) return false;
    wxVariant value;
    value << dialog.GetColourData().GetColour();
    SetValueInEvent(value);
    return true;
}
