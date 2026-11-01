// Integration regressions for the imported drawing tools and existing library.
#include "assdraw.hpp"
#include "styled_ass.hpp"

#include <wx/clipbrd.h>
#include <wx/filename.h>
#include <wx/stdpaths.h>
#include <wx/wfstream.h>
#include <cassert>
#include <iostream>

class ASSDrawTestApp : public wxApp {
public:
    bool OnInit() override { return true; }
};
wxIMPLEMENT_APP_NO_MAIN(ASSDrawTestApp);

int main(int argc, char** argv)
{
    if (!wxEntryStart(argc, argv))
        return 1;
    wxTheApp->SetAppName(wxString::Format("ASSDraw3-tests-%lu", wxGetProcessId()));
    wxTheApp->CallOnInit();
    const wxString directory = wxStandardPaths::Get().GetUserDataDir();
    wxFileName::Mkdir(directory, wxS_DIR_DEFAULT, wxPATH_MKDIR_FULL);
    const wxString config_path = wxFileName(directory, "ASSDraw3.cfg").GetFullPath();
    {
        wxFileConfig config;
        config.SetPath("settings");
        config.Write("behaviors.nosplashscreen", true);
        config.Write("behaviors.confirmquit", false);
        wxFileOutputStream output(config_path);
        config.Save(output);
    }

    auto* frame = new ASSDrawFrame(wxTheApp, TITLE, wxDefaultPosition, wxSize(800, 600));
    wxTheApp->SetTopWindow(frame);
    auto* canvas = frame->m_canvas;
    wxTheApp->Yield(true);

    // Multiline text keeps adjacent command tokens separate.
    canvas->ParseASS("m 10 10\nl 100 10\tl 100 100\r\nl 10 100");
    assert(canvas->GenerateDrawingASS().Find("100 100") != wxNOT_FOUND);

    const wxString styled =
        "{\\p1\\bord0\\1c&H0000FF&\\1a&H00&}m 10 10 l 100 10 l 100 100 l 10 100 "
        "{\\p1\\bord0\\1c&HFF0000&\\1a&H00&}m 110 10 l 200 10 l 200 100 l 110 100";
    canvas->ParseASS(styled);
    const wxString before = canvas->GenerateASS();
    const StyledAssImport logical = ImportStyledAss(std::string(before.utf8_str()), ShapeStyle());
    assert(logical.logical_runs.size() == 2);
    assert(logical.logical_runs[1].commands.front().points.front().x == 200);

    // Applying unrelated guide/background preferences must preserve every color.
    frame->ApplySettings();
    assert(canvas->GenerateASS() == before);

    // Library saves, copies, and reloads retain colors and uncompensated geometry.
    auto* preview = frame->shapelib->AddShapePreview(before);
    assert(preview->GenerateASS() == before);
    frame->shapelib->LoadToCanvas(preview);
    assert(canvas->GenerateASS() == before);

    // Both transformation modes used to dereference the previous render path.
    canvas->SetDrawMode(MODE_SCALEROTATE);
    canvas->Refresh();
    canvas->Update();
    canvas->SetDrawMode(MODE_NUT_BILINEAR);
    canvas->Refresh();
    canvas->Update();
    canvas->SetDrawMode(MODE_ARR);

    // Drawing undo/redo must leave a moved tracing image in place.
    wxImage background(20, 20);
    background.SetRGB(wxRect(0, 0, 20, 20), 128, 128, 128);
    canvas->SetBackgroundImage(background, "<clipboard>", false);
    canvas->MoveCanvasBackground(11, 17);
    unsigned width, height;
    wxRealPoint displacement;
    double scale;
    assert(canvas->GetBackgroundInfo(width, height, displacement, scale));
    canvas->ParseASS("m 0 0 l 50 0 l 50 50", true);
    assert(canvas->Undo());
    assert(canvas->GenerateASS() == before);
    wxRealPoint after;
    double after_scale;
    assert(canvas->GetBackgroundInfo(width, height, after, after_scale));
    assert(after == displacement && after_scale == scale);
    assert(canvas->Redo());
    assert(canvas->GetBackgroundInfo(width, height, after, after_scale));
    assert(after == displacement && after_scale == scale);

    // Pan toggles must work through the toolbar and the matching menu command.
    const bool old_pan = canvas->GetDragMode().bgimg;
    wxCommandEvent toolbar(wxEVT_TOOL, DRAG_BGIMG);
    frame->GetEventHandler()->ProcessEvent(toolbar);
    assert(canvas->GetDragMode().bgimg != old_pan);
    wxCommandEvent menu(wxEVT_MENU, DRAG_BGIMG);
    frame->GetEventHandler()->ProcessEvent(menu);
    assert(canvas->GetDragMode().bgimg == old_pan);

    frame->srctxtctrl->ChangeValue("{\\p1\\1c&hff00aa&}m 0 0\nl 10 10\tl 0 10");
    wxCommandEvent copy(wxEVT_BUTTON, wxID_COPY);
    frame->OnSelect_CopyCommands(copy);
    assert(wxTheClipboard->Open());
    wxTextDataObject copied;
    assert(wxTheClipboard->GetData(copied));
    wxTheClipboard->Close();
    assert(copied.GetText().Find("&HFF00AA&") != wxNOT_FOUND);
    assert(copied.GetText().Find('\n') == wxNOT_FOUND);
    assert(copied.GetText().Find('\t') == wxNOT_FOUND);

    frame->shapelib->AddShapePreview(before);
    wxCommandEvent check(wxEVT_TOOL, TOOL_CHECK);
    frame->shapelib->CheckUncheckAllPreviews(check);
    wxCommandEvent remove(wxEVT_TOOL, TOOL_DELETE);
    frame->shapelib->DeleteChecked(remove);
    assert(frame->shapelib->GetShapePreviews().empty());
    wxTheApp->Yield(true);

    frame->Destroy();
    wxTheApp->Yield(true);
    wxTheApp->OnExit();
    wxEntryCleanup();
    wxFileName::Rmdir(directory, wxPATH_RMDIR_RECURSIVE);
    std::cout << "Canvas, styling, library, transforms, clipboard, and history passed.\n";
    return 0;
}
