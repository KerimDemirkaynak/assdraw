#pragma once

#include <wx/bitmap.h>
#include <wx/bmpbndl.h>
#include <wx/colour.h>
#include <wx/string.h>

class wxWindow;
class wxMenu;

namespace AssdrawTheme {
wxString ReadPreference();
const wxString& Requested();
bool SetAppearance(const wxString& mode);
void PreparePalette();
bool IsDark();
wxColour Background();
wxColour Surface();
wxColour Text();
void ApplyControls(wxWindow* window, bool refresh_icons = true);
void ApplyMenu(wxMenu* menu);
}
