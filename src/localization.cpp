#include "localization.hpp"
#include "theme.hpp"

#include <memory>
#include <wx/fileconf.h>
#include <wx/filename.h>
#include <wx/stdpaths.h>
#include <wx/translation.h>
#include <wx/wfstream.h>

namespace AssdrawI18n {
namespace {
wxString requested = "auto";
wxString effective = "en";
[[maybe_unused]] const char* const stock_labels[] = {
    wxTRANSLATE("Yes"), wxTRANSLATE("No"), wxTRANSLATE("&Yes"), wxTRANSLATE("&No"),
    wxTRANSLATE("Close"), wxTRANSLATE("Custom"), wxTRANSLATE("Custom colour"),
    wxTRANSLATE("Basic colours"), wxTRANSLATE("Custom colours"), wxTRANSLATE("Add to custom colours")
};

wxString SupportedCode(wxString code)
{
    code.Replace("-", "_");
    if (code.StartsWith("zh_TW") || code.StartsWith("zh_HK") ||
        code.StartsWith("zh_MO") || code.StartsWith("zh_Hant"))
        return "zh_TW";
    const wxString base = code.BeforeFirst('_');
    for (const auto& language : Languages())
        if (base == language.code)
            return base;
    return "en";
}

wxString HtmlEscape(wxString text)
{
    text.Replace("&", "&amp;");
    text.Replace("<", "&lt;");
    text.Replace(">", "&gt;");
    text.Replace("\"", "&quot;");
    return text;
}
}

const std::array<Language, 7>& Languages()
{
    static const std::array<Language, 7> languages {{
        {"en", "English"}, {"tr", u8"Türkçe"}, {"fr", u8"Français"},
        {"de", "Deutsch"}, {"es", u8"Español"},
        {"zh_TW", u8"繁體中文"}, {"vi", u8"Tiếng Việt"}
    }};
    return languages;
}

wxString ConfigPath()
{
    return wxFileName(wxStandardPaths::Get().GetUserDataDir(), "ASSDraw3.cfg").GetFullPath();
}

wxString ReadLanguagePreference()
{
    if (!wxFileExists(ConfigPath()))
        return "auto";
    wxFileInputStream input(ConfigPath());
    if (!input.IsOk())
        return "auto";
    wxFileConfig config(input);
    wxString code;
    if (config.Read("/ui/language", &code))
        return code;
    long legacy = wxLANGUAGE_DEFAULT;
    if (config.Read("/Language", &legacy) && legacy != wxLANGUAGE_DEFAULT) {
        const wxLanguageInfo* info = wxLocale::GetLanguageInfo(static_cast<int>(legacy));
        if (info)
            return SupportedCode(info->CanonicalName);
    }
    return "auto";
}

wxString RequestedLanguage() { return requested; }
wxString EffectiveLanguage() { return effective; }

bool SaveConfig(wxFileConfig& config, const wxString& path)
{
    const wxString temporary = path + ".tmp";
    bool saved = false;
    {
        wxFileOutputStream output(temporary);
        saved = output.IsOk() && config.Save(output) && output.Close();
    }
    if (saved && wxRenameFile(temporary, path, true))
        return true;
    if (wxFileExists(temporary))
        wxRemoveFile(temporary);
    return false;
}

bool SetLanguage(const wxString& code)
{
    bool supported = code == "auto";
    for (const auto& language : Languages())
        supported = supported || code == language.code;
    if (!supported)
        return false;

    wxString resolved = code;
    if (code == "auto") {
        const wxLanguageInfo* info = wxLocale::GetLanguageInfo(wxLocale::GetSystemLanguage());
        resolved = info ? SupportedCode(info->CanonicalName) : wxString("en");
    }
    const wxString executable_dir = wxFileName(wxStandardPaths::Get().GetExecutablePath()).GetPath();
    wxFileTranslationsLoader::AddCatalogLookupPathPrefix(executable_dir + "/locale");
    wxFileTranslationsLoader::AddCatalogLookupPathPrefix(executable_dir + "/../share/locale");

    // Catalog selection is independent of the OS locale and of numeric parsing.
    // A fresh manager prevents an explicit English choice falling back to Turkish.
    auto next = std::make_unique<wxTranslations>();
    next->SetLanguage(resolved);
    if (resolved != "en") {
        if (!next->AddStdCatalog() || !next->IsLoaded("wxstd"))
            return false;
        // Application terminology takes precedence over standard widget labels.
        if (!next->AddCatalog("assdraw") || !next->IsLoaded("assdraw"))
            return false;
    }
    wxTranslations::Set(next.release());
    requested = code;
    effective = resolved;
    return true;
}

wxString ManualPath()
{
    const wxString directory = wxFileName(wxStandardPaths::Get().GetExecutablePath()).GetPath();
    // Downloaded CHM files can open with an empty content pane under Windows'
    // Internet Zone restrictions. The complete illustrated HTML manual works
    // from an extracted portable package as well as an installed copy.
    const wxString html = directory + "/help/index.html";
    if (wxFileExists(html))
        return html;
#ifdef __WXMSW__
    const wxString chm = wxFileName(directory, "ASSDraw3.chm").GetFullPath();
    if (wxFileExists(chm))
        return chm;
#endif
    return wxString();
}

wxString AboutHtml()
{
    const wxString bg = AssdrawTheme::Background().GetAsString(wxC2S_HTML_SYNTAX);
    const wxString fg = AssdrawTheme::Text().GetAsString(wxC2S_HTML_SYNTAX);
    const wxString link = AssdrawTheme::IsDark() ? "#80bfff" : "#175ab5";
    wxString html = "<html><body bgcolor=\"" + bg + "\" text=\"" + fg + "\" link=\"" + link + "\">";
    const auto paragraph = [&html](const wxString& text) { html += "<p>" + HtmlEscape(text) + "</p>"; };
    paragraph(_("ASSDraw3 is a tool for designing vector shapes for ASS subtitle files."));
    paragraph(_("This version was created by Kerim Demirkaynak."));
    html += "<p><a href=\"https://fontawesome.com/\">Font Awesome Free 7.2.0</a>"
        " — Fonticons, Inc. — <a href=\"https://creativecommons.org/licenses/by/4.0/\">CC BY 4.0</a></p>";
    paragraph(_("Original application: ai-chan (2006-2007)."));
    paragraph(_("Select a drawing tool, then click or drag to add lines and curves. Drag points and control handles to modify the drawing."));
    paragraph(_("Tips and shortcuts"));
    paragraph(_("Drag an image onto the canvas to use it as a tracing background."));
    paragraph(_("Use the Shapes Library to store drawings. Ctrl+Z undoes a drawing edit; Ctrl+Y redoes it."));
    paragraph(_("Use the mouse wheel or PageUp/PageDown to zoom. Right-drag pans the enabled drawing or background layer."));
    paragraph(_("Right-double-click opens a context menu. Hold Shift to temporarily switch to Drag mode."));
    paragraph(_("In Coloring mode, select a filled sub-shape and double-click it to choose its color. Copy ASS copies the styled drawing."));
    paragraph(_("Acknowledgements"));
    paragraph(_("Maxim Shemanarev: Anti-Grain Geometry. The wxWidgets team: GUI toolkit. Adrian Secord: wxAGG."));
    paragraph(_("Thanks to jfs, ArchMageZeratul, RoRo, and the Aegisub community for their suggestions and support."));
    html += "<p><a href=\"https://github.com/KerimDemirkaynak/assdraw\">ASSDraw3</a> | "
            "<a href=\"https://aegisub.org\">Aegisub</a></p></body></html>";
    return html;
}
}
