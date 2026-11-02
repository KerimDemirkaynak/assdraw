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
#include <wx/settings.h>
#include <wx/mstream.h>
#include <algorithm>
#include <array>
#include <cassert>
#include <iostream>
#include <clocale>
#include <cmath>
#include <cstdlib>

#ifdef __WXGTK3__
#include <gtk/gtk.h>
#endif

#ifdef __WXMSW__
#include <wx/msw/wrapwin.h>
#endif

static wxImage CaptureWindow(wxWindow* window)
{
    const wxSize size = window->GetClientSize();
    wxBitmap bitmap(size.x, size.y);
    wxMemoryDC output(bitmap);
    wxClientDC input(window);
    assert(output.Blit(0, 0, size.x, size.y, &input, 0, 0));
    output.SelectObject(wxNullBitmap);
    return bitmap.ConvertToImage();
}

static void CheckReferencePixels(ASSDrawCanvas* canvas, const wxImage& source,
    const wxColour& background, int opacity, int dx = 0, int dy = 0, double scale = 1.0)
{
    static unsigned check_number = 0;
    ++check_number;
    const std::array<wxPoint, 4> samples = {wxPoint(20, 20), wxPoint(100, 20),
        wxPoint(20, 60), wxPoint(100, 60)};
    wxImage actual;
    wxStopWatch elapsed;
    bool matches = false;
    do {
        // Exercise fresh native allocations between image preparation and use.
        wxBitmap churn(121, 81, 32);
        canvas->Refresh(false);
        canvas->Update();
        wxTheApp->Yield(true);
        actual = CaptureWindow(canvas);
        matches = true;
        for (const auto& point : samples) {
            int a = source.HasAlpha() ? source.GetAlpha(point.x, point.y) : 255;
            if (source.HasMask() && source.GetRed(point.x, point.y) == source.GetMaskRed()
                && source.GetGreen(point.x, point.y) == source.GetMaskGreen()
                && source.GetBlue(point.x, point.y) == source.GetMaskBlue()) a = 0;
            a = (a * opacity + 127) / 255;
            const int expected[] = {
                (source.GetRed(point.x, point.y) * a + background.Red() * (255 - a) + 127) / 255,
                (source.GetGreen(point.x, point.y) * a + background.Green() * (255 - a) + 127) / 255,
                (source.GetBlue(point.x, point.y) * a + background.Blue() * (255 - a) + 127) / 255};
            const int x = dx + int(point.x * scale), y = dy + int(point.y * scale);
            matches = matches && std::abs(int(actual.GetRed(x, y)) - expected[0]) <= 3
                && std::abs(int(actual.GetGreen(x, y)) - expected[1]) <= 3
                && std::abs(int(actual.GetBlue(x, y)) - expected[2]) <= 3;
        }
        if (matches) break;
        wxMilliSleep(10);
    } while (elapsed.Time() < 2000);
    if (!matches) {
        unsigned width = 0, height = 0;
        wxRealPoint displacement;
        double image_scale = 0;
        const bool has_image = canvas->GetBackgroundInfo(width, height, displacement, image_scale);
        std::cerr << "Reference pixels failed at check " << check_number
            << ", opacity=" << opacity << ", canvas=" << canvas->GetClientSize().x
            << "x" << canvas->GetClientSize().y << ", dpi=" << canvas->GetDPIScaleFactor()
            << ", image=" << has_image << " " << width << "x" << height
            << ", position=" << displacement.x << "," << displacement.y
            << ", scale=" << image_scale << ", background="
            << background.GetAsString(wxC2S_CSS_SYNTAX).ToStdString() << '\n';
        for (const auto& point : samples) {
            const int x = dx + int(point.x * scale), y = dy + int(point.y * scale);
            std::cerr << "  pixel " << x << "," << y << ": actual="
                << int(actual.GetRed(x, y)) << "," << int(actual.GetGreen(x, y))
                << "," << int(actual.GetBlue(x, y)) << "; source="
                << int(source.GetRed(point.x, point.y)) << ","
                << int(source.GetGreen(point.x, point.y)) << ","
                << int(source.GetBlue(point.x, point.y)) << "; alpha="
                << (source.HasAlpha() ? int(source.GetAlpha(point.x, point.y)) : 255) << '\n';
        }
        actual.SaveFile("reference-image-failure.png", wxBITMAP_TYPE_PNG);
    }
    assert(matches);
}

static void CheckClipboardImages(ASSDrawFrame* frame)
{
    auto* canvas = frame->m_canvas;
    wxImage image(121, 81); // Odd width also catches native row-padding mistakes.
    image.SetRGB(wxRect(0, 0, 121, 40), 220, 40, 60);
    image.SetRGB(wxRect(0, 40, 121, 41), 30, 80, 210);
    const long saved_opacity = frame->alphas.dfltimgopac;
    const bool saved_ask = frame->behaviors.autoaskimgopac;
    frame->alphas.dfltimgopac = 255;
    frame->behaviors.autoaskimgopac = true;
    for (int depth : {24, 32}) {
        {
            wxClipboardLocker clipboard;
            assert(!!clipboard);
            assert(wxTheClipboard->SetData(new wxBitmapDataObject(wxBitmap(image, depth))));
        }
        frame->_Paste();
        auto* slider = wxDynamicCast(wxWindow::FindWindowById(TB_BGALPHA_SLIDER, canvas), wxSlider);
        assert(slider && slider->GetParent()->IsShown());
        slider->GetParent()->Hide();
        CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
        for (int opacity : {0, 50, 100}) {
            slider->SetValue(opacity);
            wxScrollEvent event(wxEVT_SCROLL_THUMBTRACK, TB_BGALPHA_SLIDER, opacity);
            slider->GetEventHandler()->ProcessEvent(event);
            CheckReferencePixels(canvas, image, frame->colors.canvas_bg,
                int(std::lround(opacity * 2.55)));
        }
    }
    frame->behaviors.autoaskimgopac = false;
    image.InitAlpha();
    for (int y = 0; y < image.GetHeight(); ++y)
        for (int x = 0; x < image.GetWidth(); ++x)
            image.SetAlpha(x, y, x < 60 ? (y < 40 ? 255 : 0) : (y < 40 ? 128 : 64));
#ifdef __WXMSW__
    wxMemoryOutputStream png;
    assert(image.SaveFile(png, wxBITMAP_TYPE_PNG));
    auto* data = new wxCustomDataObject(wxDataFormat("PNG"));
    std::vector<unsigned char> bytes(png.GetSize());
    png.CopyTo(bytes.data(), bytes.size());
    assert(data->SetData(bytes.size(), bytes.data()));
#else
    // wxGTK represents its standard bitmap clipboard format as image/png.
    auto* data = new wxBitmapDataObject(wxBitmap(image));
#endif
    {
        wxClipboardLocker clipboard;
        assert(!!clipboard && wxTheClipboard->SetData(data));
    }
    frame->_Paste();
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    // Repeat paints and change image transforms after the clipboard owner and
    // all its temporary bitmap objects have been destroyed.
    canvas->MoveCanvasBackground(15, 19);
    canvas->ChangeBackgroundZoomLevel(1.5, wxRealPoint(0, 0));
    unsigned width, height;
    wxRealPoint displacement;
    double scale;
    assert(canvas->GetBackgroundInfo(width, height, displacement, scale));
    assert(width == 121 && height == 81 && scale == 1.5);
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255,
        int(displacement.x), int(displacement.y), scale);
    canvas->SetBackgroundImage(wxImage(), "invalid", false);
    assert(canvas->GetBackgroundInfo(width, height, displacement, scale));
    assert(width == 121 && height == 81); // Invalid input keeps the previous image.
    canvas->SetBackgroundImage(image, "alpha", false);
    for (const char* theme : {"dark", "light"}) {
        assert(frame->ApplyTheme(theme));
        wxTheApp->Yield(true);
        frame->RefreshTheme();
        CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    }
    image.ClearAlpha();
    image.SetMaskColour(220, 40, 60);
    canvas->SetBackgroundImage(image, "mask", false);
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    // Source mutation must not alter the retained tracing image.
    wxImage changed = image.Copy();
    canvas->SetBackgroundImage(changed, "owned", false);
    changed.SetRGB(wxRect(0, 0, 121, 81), 0, 255, 0);
    canvas->PrepareBackgroundBitmap(-1.0);
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    const wxString filename = wxFileName::CreateTempFileName("assdraw-reference-");
    assert(image.SaveFile(filename, wxBITMAP_TYPE_PNG));
    canvas->ReceiveBackgroundImageFileDropEvent(filename);
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    assert(wxRemoveFile(filename));
    {
        wxLogNull suppress_missing_file;
        canvas->ReceiveBackgroundImageFileDropEvent(filename);
    }
    assert(frame->GetStatusBar()->GetStatusText(1).empty());
    CheckReferencePixels(canvas, image, frame->colors.canvas_bg, 255);
    canvas->RemoveBackgroundImage();
    assert(!canvas->HasBackgroundImage());
    frame->alphas.dfltimgopac = saved_opacity;
    frame->behaviors.autoaskimgopac = saved_ask;
}

static void CheckAboutBanner(ASSDrawFrame* frame)
{
    wxImage expected(wxString::FromUTF8(ASSDRAW_SOURCE_DIR) + "/src/bitmaps/assdraw3.bmp", wxBITMAP_TYPE_BMP);
    assert(expected.IsOk());
    auto* about = new ASSDrawAboutDlg(frame, 0);
    BigStaticBitmapCtrl* banner = nullptr;
    for (wxWindow* child : about->GetChildren())
        if ((banner = dynamic_cast<BigStaticBitmapCtrl*>(child))) break;
    assert(banner && banner->GetSize() == banner->FromDIP(expected.GetSize()));
    about->Show();
    wxTheApp->Yield(true);
    banner->Update();
    const wxImage actual = CaptureWindow(banner);
    expected.Rescale(actual.GetWidth(), actual.GetHeight(), wxIMAGE_QUALITY_HIGH);
    // Compare the rendered banner, not just metadata or presence of a resource.
    size_t matches = 0;
    const size_t count = size_t(actual.GetWidth()) * actual.GetHeight() * 3;
    for (size_t i = 0; i < count; ++i)
        if (std::abs(int(actual.GetData()[i]) - expected.GetData()[i]) <= 5) ++matches;
    assert(matches > count * 0.95);
    about->Destroy();
    wxTheApp->Yield(true);
}

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

static void CheckVectorIcons()
{
    const wxColour foreground = AssdrawTheme::Text();
    for (int icon = 0; icon < static_cast<int>(AssdrawIcons::Icon::Count); ++icon) {
        const auto bundle = AssdrawIcons::Bitmap(static_cast<AssdrawIcons::Icon>(icon));
        assert(bundle.IsOk());
        for (int size : {20, 40, 80}) {
            const wxBitmap bitmap = bundle.GetBitmap(wxSize(size, size));
            assert(bitmap.GetSize() == wxSize(size, size));
            const wxImage image = bitmap.ConvertToImage();
            assert(image.HasAlpha() && image.GetAlpha(0, 0) == 0);
            int ink = 0;
            for (int y = 0; y < size; ++y) {
                for (int x = 0; x < size; ++x) {
                    if (image.GetAlpha(x, y) < 128) continue;
                    assert(std::abs(int(image.GetRed(x, y)) - foreground.Red()) <= 2);
                    assert(std::abs(int(image.GetGreen(x, y)) - foreground.Green()) <= 2);
                    assert(std::abs(int(image.GetBlue(x, y)) - foreground.Blue()) <= 2);
                    ++ink;
                }
            }
            if (ink <= 5) {
                std::cerr << "Empty SVG icon=" << icon << " size=" << size << '\n';
                image.SaveFile(wxString::Format("theme-icon-%d-%d.png", icon, size), wxBITMAP_TYPE_PNG);
            }
            assert(ink > 5);
        }
    }
}

static void CheckToolbarPositions(ASSDrawFrame* frame)
{
    const std::array<wxToolBar*, 3> bars = {frame->drawtbar, frame->modetbar, frame->bgimgtbar};
    for (size_t i = 0; i < bars.size(); ++i) {
        if (!bars[i]->IsShownOnScreen()) continue;
        const wxRect first = bars[i]->GetScreenRect();
        for (size_t j = i + 1; j < bars.size(); ++j) {
            if (!bars[j]->IsShownOnScreen()) continue;
            const wxRect second = bars[j]->GetScreenRect();
            if (first.Intersects(second))
                std::cerr << "Overlapping toolbars " << i << " and " << j << std::endl;
            assert(!first.Intersects(second));
        }
    }
}

static void CheckLibraryResize(ASSDrawFrame* frame, ASSDrawShapePreview* drawing)
{
    auto* library = frame->shapelib;
    const wxString commands = drawing->GenerateASS();
    // Match the reported case with several previews and a vertical scrollbar.
    for (int i = 0; i < 3; ++i)
        library->AddShapePreview("");
    // Native toolbar metrics differ between GTK and Windows. Measure the
    // legitimate minimum once, before any widening, then insist that all
    // later shrink operations can return to this original bound.
    const int original_minimum = library->libsizer->GetMinSize().x;
    const int narrow_width = std::max(frame->FromDIP(160), original_minimum + frame->FromDIP(8));
    assert(narrow_width < frame->FromDIP(300));
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
            const wxSize requested(std::max(narrow_width, frame->FromDIP(width)), frame->FromDIP(500));
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
                std::cerr << "Library resize stuck: requested=" << requested.x
                    << ", original_minimum=" << original_minimum
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
    bool OnInit() override { wxInitAllImageHandlers(); return AssdrawTheme::SetAppearance("light"); }
};
wxIMPLEMENT_APP_NO_MAIN(ASSDrawTestApp);

int main(int argc, char** argv)
{
    if (!wxEntryStart(argc, argv))
        return 1;
    wxSetAssertHandler([](const wxString& file, int line, const wxString& function,
        const wxString& condition, const wxString& message) {
        std::cerr << file.ToStdString() << ':' << line << ' ' << function.ToStdString()
            << ": " << condition.ToStdString() << ' ' << message.ToStdString() << std::endl;
        std::abort();
    });
    wxTheApp->SetAppName(wxString::Format("ASSDraw3-tests-%lu", wxGetProcessId()));
    assert(wxTheApp->CallOnInit());
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
    CheckClipboardImages(frame);
    CheckAboutBanner(frame);

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

    // Exercise the actual wxWidgets native appearance API, including controls
    // created before the switch, and keep the document, image and pending edits.
    const wxString theme_document = canvas->GenerateASS();
    const wxString theme_pending = frame->srctxtctrl->GetValue();
    const wxString theme_undo = canvas->GetTopUndo();
    const wxString theme_redo = canvas->GetTopRedo();
    const wxColour light_canvas = frame->colors.canvas_bg;
    const wxColour light_library = frame->colors.library_libarea;
#ifdef __WXGTK3__
    gchar* initial_gtk_name = nullptr;
    g_object_get(gtk_settings_get_default(), "gtk-theme-name", &initial_gtk_name, nullptr);
    const wxString light_gtk_theme = wxString::FromUTF8(initial_gtk_name);
    g_free(initial_gtk_name);
#endif
    frame->settingsdlg->propgrid->SetPropertyValue(frame->settingsdlg->alphas_dfltimgopac_pgid, 77L);
    wxColour pending_origin(10, 120, 190);
    frame->settingsdlg->propgrid->SetPropertyValue(frame->settingsdlg->colors_origin_pgid, pending_origin);
    frame->settingsdlg->colors_origin_pgid->SetModifiedStatus(true);
    const auto settle_theme = [frame](bool dark) {
        wxStopWatch elapsed;
        while (AssdrawTheme::IsDark() != dark && elapsed.Time() < 2000) {
            wxTheApp->Yield(true);
            wxMilliSleep(10);
        }
        wxTheApp->Yield(true);
        frame->RefreshTheme();
        if (AssdrawTheme::IsDark() != dark) {
            std::cerr << "Theme did not settle: requested=" << dark
                << " background=" << AssdrawTheme::Background().GetAsString().ToStdString()
                << " text=" << AssdrawTheme::Text().GetAsString().ToStdString() << '\n';
#ifdef __WXGTK3__
            gchar* name = nullptr;
            gboolean prefer_dark = FALSE;
            g_object_get(gtk_settings_get_default(), "gtk-theme-name", &name,
                "gtk-application-prefer-dark-theme", &prefer_dark, nullptr);
            std::cerr << "GTK theme=" << (name ? name : "") << " prefer-dark=" << prefer_dark << '\n';
            g_free(name);
#endif
        }
        assert(AssdrawTheme::IsDark() == dark);
        assert(frame->view_is_dark == dark);
    };
    const auto snapshot = [](wxWindow* window, const char* name) {
        window->Refresh(); window->Update(); wxTheApp->Yield(true);
        const wxSize size = window->GetSize();
        wxBitmap bitmap(size.x, size.y);
        wxMemoryDC target(bitmap);
        wxWindowDC source(window);
        assert(target.Blit(0, 0, size.x, size.y, &source, 0, 0));
        target.SelectObject(wxNullBitmap);
        assert(bitmap.SaveFile(wxString::FromUTF8(name), wxBITMAP_TYPE_PNG));
    };
    for (int repeat = 0; repeat < 3; ++repeat) {
        assert(frame->ApplyTheme("dark"));
        settle_theme(true);
        CheckToolbarPositions(frame);
        assert(frame->themeMenu->IsChecked(6102));
        assert(AssdrawTheme::ReadPreference() == "dark");
        assert(frame->colors.canvas_bg.Red() < 100);
        assert(frame->srctxtctrl->GetBackgroundColour() == AssdrawTheme::Background());
        assert(frame->drawtbar->GetBackgroundColour() == AssdrawTheme::Surface());
        assert(frame->zoomslider->GetBackgroundColour() == AssdrawTheme::Surface());
#ifdef __WXMSW__
        assert(frame->drawtbar->IsDoubleBuffered());
        // Both prebuilt and freshly created popup menus have their erase brush
        // set before opening; otherwise their first frame uses COLOR_MENU.
        wxMenu popup;
        popup.Append(wxID_ANY, "Test");
        AssdrawTheme::ApplyMenu(&popup);
        for (wxMenu* menu : {frame->drawMenu, &popup}) {
            MENUINFO info{};
            info.cbSize = sizeof(info);
            info.fMask = MIM_BACKGROUND;
            assert(::GetMenuInfo(static_cast<HMENU>(menu->GetHMenu()), &info));
            assert(info.hbrBack);
            LOGBRUSH brush{};
            assert(::GetObject(info.hbrBack, sizeof(brush), &brush));
            assert(GetRValue(brush.lbColor) < 100 && GetGValue(brush.lbColor) < 100
                && GetBValue(brush.lbColor) < 100);
        }
#endif
        assert(frame->settingsdlg->propgrid->GetCellBackgroundColour() == AssdrawTheme::Background());
        assert(frame->settingsdlg->propgrid->GetPropertyValueAsLong(frame->settingsdlg->alphas_dfltimgopac_pgid) == 77);
        wxColour origin_value;
        origin_value << frame->settingsdlg->propgrid->GetPropertyValue(frame->settingsdlg->colors_origin_pgid);
        assert(origin_value == pending_origin);
        assert(canvas->GenerateASS() == theme_document);
        assert(frame->srctxtctrl->GetValue() == theme_pending);
        assert(canvas->GetTopUndo() == theme_undo && canvas->GetTopRedo() == theme_redo);
        assert(canvas->GetBackgroundInfo(width, height, after, after_scale));
        assert(after == displacement && after_scale == scale);
        if (repeat == 0) {
            snapshot(frame, "theme-dark.png");
            auto* about = new ASSDrawAboutDlg(frame, 0);
            about->Show();
            wxTheApp->Yield(true);
            assert(about->GetBackgroundColour() == AssdrawTheme::Surface());
            snapshot(about, "theme-dark-about.png");
            about->Destroy();
            wxTheApp->Yield(true);
            auto* transform = new ASSDrawTransformDlg(frame);
            transform->Show();
            wxTheApp->Yield(true);
            snapshot(transform, "theme-dark-transform.png");
            transform->Destroy();
            wxTheApp->Yield(true);
        }
        CheckVectorIcons();
        assert(frame->ApplyTheme("light"));
        settle_theme(false);
        CheckToolbarPositions(frame);
        CheckVectorIcons();
        assert(frame->themeMenu->IsChecked(6101));
        assert(frame->colors.canvas_bg == light_canvas);
        assert(frame->colors.library_libarea == light_library);
#ifdef __WXGTK3__
        gchar* restored_gtk_name = nullptr;
        g_object_get(gtk_settings_get_default(), "gtk-theme-name", &restored_gtk_name, nullptr);
        assert(wxString::FromUTF8(restored_gtk_name) == light_gtk_theme);
        g_free(restored_gtk_name);
#endif
        assert(canvas->GenerateASS() == theme_document);
        assert(frame->srctxtctrl->GetValue() == theme_pending);
        if (repeat == 0) snapshot(frame, "theme-light.png");
    }
    assert(frame->ApplyTheme("dark")); settle_theme(true);
    const wxColour custom_dark(20, 30, 40);
    frame->colors.canvas_bg = custom_dark;
    frame->ApplyViewColours();
    assert(frame->ApplyTheme("light")); settle_theme(false);
    assert(frame->colors.canvas_bg == light_canvas);
    assert(frame->ApplyTheme("dark")); settle_theme(true);
    assert(frame->colors.canvas_bg == custom_dark);
    assert(frame->ApplyTheme("system"));
    wxTheApp->Yield(true);
    assert(frame->themeMenu->IsChecked(6100));
    assert(AssdrawTheme::ReadPreference() == "system");
    assert(!frame->ApplyTheme("invalid-theme"));
    assert(frame->ApplyTheme("light")); settle_theme(false);
    frame->settingsdlg->RefreshSettingsDisplay();
    frame->settingsdlg->propgrid->ClearModifiedStatus();

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
    assert(frame->ApplyTheme("dark"));
    frame->Destroy();
    wxTheApp->Yield(true);
    // The next launch selects the saved native theme before creating controls.
    assert(AssdrawTheme::ReadPreference() == "dark");
    assert(AssdrawTheme::SetAppearance(AssdrawTheme::ReadPreference()));
    frame = new ASSDrawFrame(wxTheApp, TITLE, wxDefaultPosition, wxSize(800, 600));
    wxTheApp->SetTopWindow(frame);
    wxTheApp->Yield(true);
    assert(frame->view_is_dark && frame->themeMenu->IsChecked(6102));
    assert(frame->colors.canvas_bg == custom_dark);
    frame->Destroy();
    wxTheApp->Yield(true);
    wxTheApp->OnExit();
    wxEntryCleanup();
    wxFileName::Rmdir(directory, wxPATH_RMDIR_RECURSIVE);
    std::cout << "Canvas, styling, library, transforms, native themes, language switching, catalogs, manuals, clipboard, and history passed.\n";
    return 0;
}
