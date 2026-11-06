#pragma once

#include <wx/colordlg.h>
#include <wx/propgrid/advprops.h>

// Keep the native chooser, but use the application's catalogs for its text.
class AssdrawColourDialog : public wxColourDialog
{
public:
    AssdrawColourDialog(wxWindow* parent, const wxColourData* data);
#ifdef __WXMSW__
    void MSWOnInitDone(WXHWND dialog) override;
#endif
};

class AssdrawColourProperty : public wxColourProperty
{
public:
    AssdrawColourProperty(const wxString& label, const wxString& name, const wxColour& value);
    void UpdateTranslations();
    bool OnEvent(wxPropertyGrid* grid, wxWindow* primary, wxEvent& event) override;
    bool IntToValue(wxVariant& value, int number, wxPGPropValFormatFlags flags) const override;
};
