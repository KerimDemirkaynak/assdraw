// Integration regressions for the imported drawing tools and existing library.
#include "assdraw.hpp"
#include "styled_ass.hpp"

#include <wx/clipbrd.h>
#include <wx/filename.h>
#include <wx/stdpaths.h>
#include <wx/wfstream.h>
#include <cassert>
#include <iostream>
#include <clocale>

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
    assert(AssdrawI18n::SetLanguage("en"));
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
    assert(!canvas->GetTopRedo().empty());
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

    // Changing language updates existing UI without changing document/view state.
    const wxString document = canvas->GenerateASS();
    const wxString typed_commands = frame->srctxtctrl->GetValue();
    const wxString numeric_locale = wxString::FromUTF8(std::setlocale(LC_NUMERIC, nullptr));
    const std::pair<const char*, const char*> languages[] = {
        {"tr", u8"Dil"}, {"fr", "Langue"}, {"de", "Sprache"},
        {"es", "Idioma"}, {"zh_TW", u8"語言"}, {"vi", u8"Ngôn ngữ"}, {"en", "Language"}
    };
    for (int repeat = 0; repeat < 3; ++repeat) {
        for (const auto& language : languages) {
            assert(frame->ApplyLanguage(wxString::FromUTF8(language.first)));
            wxTheApp->Yield(true);
            assert(frame->GetMenuBar()->GetMenuLabelText(5) == wxString::FromUTF8(language.second));
            assert(canvas->GenerateASS() == document);
            assert(frame->srctxtctrl->GetValue() == typed_commands);
            assert(canvas->GetBackgroundInfo(width, height, after, after_scale));
            assert(after == displacement && after_scale == scale);
            assert(AssdrawI18n::ReadLanguagePreference() == language.first);
            assert(wxString::FromUTF8(std::setlocale(LC_NUMERIC, nullptr)) == numeric_locale);
            if (wxString(language.first) != "en")
                assert(frame->bgimgtbar->FindById(DRAG_BGIMG)->GetLabel() != "Pan background");
        }
    }
    assert(frame->ApplyLanguage("tr"));
    assert(frame->bgimgtbar->FindById(DRAG_BGIMG)->GetLabel() == wxString::FromUTF8(u8"Arka planı kaydır"));
    assert(frame->settingsdlg->colors_canvas_bg_pgid->GetLabel() == "Tuval");
    assert(AssdrawI18n::AboutHtml().Contains(wxString::FromUTF8(u8"Bu sürüm Kerim Demirkaynak tarafından oluşturuldu.")));
    frame->transformdlg = new ASSDrawTransformDlg(frame);
    assert(frame->transformdlg->combo_templates->GetString(1) == wxString::FromUTF8(u8"5 birim aşağı taşı"));
    assert(frame->transformdlg->combo_templates->GetString(3) == wxString::FromUTF8(u8"(1, 2) etrafında saat yönünde 90° döndür"));
    frame->transformdlg->combo_templates->SetSelection(1);
    wxCommandEvent select_template(wxEVT_COMBOBOX);
    frame->transformdlg->OnTemplatesCombo(select_template);
    assert(frame->transformdlg->txtctrl_ny->GetValue() == "5.0");

    // Exercise the actual deferred menu event and verify Turkish -> English.
    wxCommandEvent english(wxEVT_MENU, 6001);
    frame->GetEventHandler()->ProcessEvent(english);
    wxTheApp->Yield(true);
    assert(AssdrawI18n::EffectiveLanguage() == "en");
    assert(frame->GetMenuBar()->GetMenuLabelText(5) == "Language");
    assert(frame->bgimgtbar->FindById(DRAG_BGIMG)->GetLabel() == "Pan background");
    assert(frame->transformdlg == nullptr);
    assert(AssdrawI18n::AboutHtml().Contains("This version was created by Kerim Demirkaynak."));
    assert(!AssdrawI18n::SetLanguage("unsupported-language"));
    assert(AssdrawI18n::EffectiveLanguage() == "en");
    const wxString manual = AssdrawI18n::ManualPath();
    assert(!manual.empty() && wxFileExists(manual));
    const wxString previous_directory = wxGetCwd();
    assert(wxSetWorkingDirectory(directory));
    assert(AssdrawI18n::ManualPath() == manual);
    assert(wxSetWorkingDirectory(previous_directory));

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
    std::cout << "Canvas, styling, library, transforms, language switching, catalogs, manuals, clipboard, and history passed.\n";
    return 0;
}
