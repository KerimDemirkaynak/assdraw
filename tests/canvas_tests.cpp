// Integration regressions for the imported drawing tools and existing library.
#include "assdraw.hpp"
#include "styled_ass.hpp"

#include <wx/clipbrd.h>
#include <wx/filename.h>
#include <wx/stdpaths.h>
#include <wx/wfstream.h>
#include <wx/dcclient.h>
#include <wx/dcmemory.h>
#include <wx/iconbndl.h>
#include <wx/stopwatch.h>
#include <wx/utils.h>
#include <algorithm>
#include <cassert>
#include <iostream>
#include <clocale>
#include <cmath>

#ifdef __WXMSW__
#include <wx/msw/wrapwin.h>
#endif

static void CheckPreviewPixels(ASSDrawShapePreview* preview, const char* stage)
{
    // A serialized drawing alone doesn't prove a visible thumbnail exists.
    // Layout/expose events arrive asynchronously from the window server, so
    // wait for visible ink rather than assuming three Yield calls are enough.
    wxStopWatch elapsed;
    wxImage image;
    int ink = 0;
    do {
        preview->Refresh(false);
        preview->Update();
        wxTheApp->Yield(true);
        const PointSystem* points = preview->_PointSystem();
        assert(std::isfinite(points->scale) && points->scale > 0);
        assert(std::isfinite(points->originx) && std::isfinite(points->originy));
        const wxSize size = preview->GetClientSize();
        if (!preview->IsShownOnScreen() || size.x < 32 || size.y < 32) {
            wxMilliSleep(10);
            continue;
        }
        wxBitmap screenshot(size.x, size.y);
        wxMemoryDC output(screenshot);
        wxClientDC input(preview);
        assert(output.Blit(0, 0, size.x, size.y, &input, 0, 0));
        output.SelectObject(wxNullBitmap);
        image = screenshot.ConvertToImage();
        ink = 0;
        // Exclude the selection checkbox and the window border.
        const int checkbox_extent = preview->FromDIP(24);
        for (int y = 4; y < size.y - 4; ++y) {
            for (int x = 4; x < size.x - 4; ++x) {
                if (x < checkbox_extent && y < checkbox_extent)
                    continue;
                if (image.GetRed(x, y) < 200 && image.GetBlue(x, y) > 50)
                    ++ink;
            }
        }
        if (ink > 50)
            return;
        wxMilliSleep(10);
    } while (elapsed.Time() < 3000);
    if (image.IsOk())
        image.SaveFile("preview-" + wxString::FromUTF8(stage) + ".png", wxBITMAP_TYPE_PNG);
    const wxSize size = preview->GetClientSize();
    std::cerr << "Blank preview at " << stage << ": " << size.x << "x" << size.y
        << ", ink=" << ink << ", scale=" << preview->_PointSystem()->scale
        << ", origin=" << preview->_PointSystem()->originx << "," << preview->_PointSystem()->originy << "\n";
    assert(ink > 50);
}

static void CheckLibraryResize(ASSDrawFrame* frame, ASSDrawShapePreview* drawing)
{
    auto* library = frame->shapelib;
    const wxString commands = drawing->GenerateASS();
    // Match the reported case with several previews and a vertical scrollbar.
    for (int i = 0; i < 3; ++i)
        library->AddShapePreview("");
    const wxString perspective = frame->m_mgr.SavePerspective();
    frame->m_mgr.GetPane(library).Float().FloatingPosition(40, 40)
        .FloatingSize(frame->FromDIP(wxSize(220, 500)));
    frame->m_mgr.Update();
    wxTheApp->Yield(true);
    wxFrame* host = frame->m_mgr.GetPane(library).frame;
    assert(host);

    // Resize the real enclosing window; do not directly resize previews or
    // manually call their layout helper, which would mask a broken size event.
    for (int repeat = 0; repeat < 3; ++repeat) {
        for (int width : {220, 560, 180, 700, 160, 480, 180, 220}) {
            const wxSize requested = frame->FromDIP(wxSize(width, 500));
            host->SetClientSize(requested);
            wxStopWatch elapsed;
            bool fitted = false;
            do {
                host->Layout();
                wxTheApp->Yield(true);
                const wxSize viewport = library->libarea->GetClientSize();
                const int expected = std::max(frame->FromDIP(32), viewport.x - frame->FromDIP(10));
                fitted = std::abs(host->GetClientSize().x - requested.x) <= frame->FromDIP(4)
                    && library->GetSize().x <= host->GetClientSize().x
                    && library->libarea->GetSize().x <= library->GetClientSize().x
                    && library->libarea->GetVirtualSize().x <= viewport.x + frame->FromDIP(2);
                for (auto* preview : library->GetShapePreviews()) {
                    fitted = fitted && std::abs(preview->GetSize().x - expected) <= frame->FromDIP(2)
                        && std::abs(preview->GetSize().y - expected) <= frame->FromDIP(2);
                }
                if (fitted)
                    break;
                wxMilliSleep(10);
            } while (elapsed.Time() < 2000);
            if (!fitted) {
                std::cerr << "Library resize stuck: requested=" << width
                    << ", host=" << host->GetClientSize().x << ", library=" << library->GetSize().x
                    << ", viewport=" << library->libarea->GetClientSize().x
                    << ", virtual=" << library->libarea->GetVirtualSize().x
                    << ", preview=" << drawing->GetSize().x << "x" << drawing->GetSize().y << "\n";
            }
            assert(fitted);
            assert(drawing->GenerateASS() == commands);
        }
    }
    library->libarea->Scroll(0, 100000);
    wxTheApp->Yield(true);
    assert(library->GetShapePreviews().size() == 4);
    library->libarea->Scroll(0, 0);
    assert(frame->m_mgr.LoadPerspective(perspective));
    frame->ApplyDpiLayout();
    wxTheApp->Yield(true);
    CheckPreviewPixels(drawing, "after-library-resize");
}

class ASSDrawTestApp : public wxApp {
public:
    bool OnInit() override { wxInitAllImageHandlers(); return true; }
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
    assert(frame->m_mgr.GetPane(frame->settingsdlg).IsShown());
    assert(frame->viewMenu->IsChecked(MENU_SETTINGS));

#ifdef __WXMSW__
    // A valid title-bar icon doesn't guarantee that Windows has a taskbar icon.
    assert(frame->GetIcons().GetIconOfExactSize(16).IsOk());
    assert(frame->GetIcons().GetIconOfExactSize(32).IsOk());
    assert(frame->GetIcons().GetIconOfExactSize(48).IsOk());
    const HWND handle = static_cast<HWND>(frame->GetHandle());
    assert(::SendMessage(handle, WM_GETICON, ICON_SMALL, 0) != 0);
    const HICON big = reinterpret_cast<HICON>(::SendMessage(handle, WM_GETICON, ICON_BIG, 0));
    assert(big != nullptr);
    ICONINFO info{};
    assert(::GetIconInfo(big, &info));
    BITMAP native_bitmap{};
    assert(::GetObject(info.hbmColor, sizeof(native_bitmap), &native_bitmap));
    assert(native_bitmap.bmWidth >= 32 && native_bitmap.bmHeight >= 32);
    ::DeleteObject(info.hbmColor);
    ::DeleteObject(info.hbmMask);
#endif

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
    CheckPreviewPixels(preview, "initial");
    // Empty/single-axis drawings and a collapsed pane must not poison the
    // transform used when a stored preview is subsequently replaced.
    for (const wxString& degenerate : {wxString(""), wxString("m 0 0"), wxString("m 0 0 l 0 100")}) {
        preview->ParseASS(degenerate);
        preview->FitToViewPoint(10, 10);
        assert(std::isfinite(preview->_PointSystem()->scale));
        assert(std::isfinite(preview->_PointSystem()->originx));
        assert(std::isfinite(preview->_PointSystem()->originy));
    }
    preview->SetSize(1, 1);
    preview->FitToViewPoint(10, 10);
    preview->ParseASS(before);
    frame->shapelib->UpdatePreviewDisplays();
    CheckPreviewPixels(preview, "after-resize");
    CheckLibraryResize(frame, preview);
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
    frame->m_mgr.GetPane(frame->bgimgtbar).Float().FloatingPosition(40, 40).FloatingSize(450, 90);
    frame->m_mgr.Update();
    wxTheApp->Yield(true);
    const std::pair<const char*, const char*> languages[] = {
        {"tr", u8"Dil"}, {"fr", "Langue"}, {"de", "Sprache"},
        {"es", "Idioma"}, {"zh_TW", u8"語言"}, {"vi", u8"Ngôn ngữ"}, {"en", "Language"}
    };
    for (int repeat = 0; repeat < 3; ++repeat) {
        for (const auto& language : languages) {
            assert(frame->ApplyLanguage(wxString::FromUTF8(language.first)));
            wxTheApp->Yield(true);
            assert(frame->GetMenuBar()->GetMenuLabelText(5) == wxString::FromUTF8(language.second));
            assert(frame->m_mgr.GetPane(frame->bgimgtbar).IsFloating());
            assert(frame->m_mgr.GetPane(frame->bgimgtbar).frame != nullptr);
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
    frame->_ResetPerspective();
    assert(frame->m_mgr.GetPane(frame->commandpanel).caption == wxString::FromUTF8(u8"Çizim komutları"));
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
    assert(manual.EndsWith("help/index.html"));
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

    // Simulate an existing 3.1 workspace from before Settings was visible by
    // default. The migration must open it once, then respect a user hide.
    frame->m_mgr.GetPane(frame->settingsdlg).Hide();
    frame->m_mgr.Update();
    frame->config->DeleteEntry("/ui/settings_visible_default_applied");
    frame->Destroy();
    wxTheApp->Yield(true);
    frame = new ASSDrawFrame(wxTheApp, TITLE, wxDefaultPosition, wxSize(800, 600));
    wxTheApp->SetTopWindow(frame);
    wxTheApp->Yield(true);
    assert(frame->m_mgr.GetPane(frame->settingsdlg).IsShown());
    assert(frame->viewMenu->IsChecked(MENU_SETTINGS));
    wxCommandEvent hide_settings(wxEVT_MENU, MENU_SETTINGS);
    frame->GetEventHandler()->ProcessEvent(hide_settings);
    assert(!frame->m_mgr.GetPane(frame->settingsdlg).IsShown());
    frame->Destroy();
    wxTheApp->Yield(true);
    frame = new ASSDrawFrame(wxTheApp, TITLE, wxDefaultPosition, wxSize(800, 600));
    wxTheApp->SetTopWindow(frame);
    wxTheApp->Yield(true);
    assert(!frame->m_mgr.GetPane(frame->settingsdlg).IsShown());
    assert(!frame->viewMenu->IsChecked(MENU_SETTINGS));
    frame->_ResetPerspective();
    assert(frame->m_mgr.GetPane(frame->settingsdlg).IsShown());
    assert(frame->viewMenu->IsChecked(MENU_SETTINGS));
    frame->Destroy();
    wxTheApp->Yield(true);
    wxTheApp->OnExit();
    wxEntryCleanup();
    wxFileName::Rmdir(directory, wxPATH_RMDIR_RECURSIVE);
    std::cout << "Canvas, styling, library, transforms, language switching, catalogs, manuals, clipboard, and history passed.\n";
    return 0;
}
