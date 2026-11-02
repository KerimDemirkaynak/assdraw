#pragma once

#include <wx/bitmap.h>
#include <wx/bmpbndl.h>
#include <wx/colour.h>
#include <wx/string.h>

class wxWindow;

namespace AssdrawTheme {
wxString ReadPreference();
const wxString& Requested();
bool SetAppearance(const wxString& mode);
bool IsDark();
wxColour Background();
wxColour Surface();
wxColour Text();
wxBitmapBundle ToolBitmap(const wxBitmap& bitmap);
void ApplyControls(wxWindow* window, bool invert_tool_icons = false);
}
