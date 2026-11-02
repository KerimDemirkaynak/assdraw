#pragma once

#include <wx/bmpbndl.h>

class wxToolBar;

namespace AssdrawIcons {
enum class Icon {
    Clear, Preview, Color, Drag, Move, Line, Bezier, DeletePoint, ScaleRotate,
    Bilinear, PanDrawing, PanBackground, Save, SelectAll, SelectNone, DeleteShapes, Copy, Count
};
wxBitmapBundle Bitmap(Icon icon);
void RefreshToolbar(wxToolBar* toolbar);
}

#define ASSDRAW_ICON(name) AssdrawIcons::Bitmap(AssdrawIcons::Icon::name)
