#include "icons.hpp"
#include "theme.hpp"
#include "enums.hpp"
#include "library.hpp"
#include "assdraw_icons_data.hpp"

#include <wx/toolbar.h>
#include <utility>

namespace AssdrawIcons {
wxBitmapBundle Bitmap(Icon icon)
{
    using namespace AssdrawIconData;
    static const char* const data[] = {Clear, Preview, Color, Drag, Move, Line,
        Bezier, DeletePoint, ScaleRotate, Bilinear, PanDrawing, PanBackground,
        Save, SelectAll, SelectNone, DeleteShapes, Copy};
    static_assert(sizeof(data) / sizeof(*data) == static_cast<size_t>(Icon::Count));
    const auto index = static_cast<size_t>(icon);
    if (index >= static_cast<size_t>(Icon::Count)) return {};
    wxString svg = wxString::FromUTF8(data[index]);
    svg.Replace("currentColor", AssdrawTheme::Text().GetAsString(wxC2S_HTML_SYNTAX));
    return wxBitmapBundle::FromSVG(svg.utf8_str().data(), wxSize(20, 20));
}

void RefreshToolbar(wxToolBar* toolbar)
{
    const std::pair<int, Icon> icons[] = {
        {TB_CLEAR, Icon::Clear}, {TB_PREVIEW, Icon::Preview}, {MODE_COLOR, Icon::Color},
        {MODE_ARR, Icon::Drag}, {MODE_M, Icon::Move}, {MODE_L, Icon::Line},
        {MODE_B, Icon::Bezier}, {MODE_DEL, Icon::DeletePoint},
        {MODE_SCALEROTATE, Icon::ScaleRotate}, {MODE_NUT_BILINEAR, Icon::Bilinear},
        {DRAG_DWG, Icon::PanDrawing}, {DRAG_BGIMG, Icon::PanBackground},
        {TOOL_SAVE, Icon::Save}, {TOOL_CHECK, Icon::SelectAll},
        {TOOL_UNCHECK, Icon::SelectNone}, {TOOL_DELETE, Icon::DeleteShapes}
    };
    for (const auto& item : icons)
        if (toolbar->FindById(item.first))
            toolbar->SetToolNormalBitmap(item.first, Bitmap(item.second));
    toolbar->Realize();
}
}
