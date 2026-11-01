#pragma once

#include <array>
#include <wx/intl.h>
#include <wx/fileconf.h>

namespace AssdrawI18n {
struct Language {
    const char* code;
    const char* native_name;
};

const std::array<Language, 7>& Languages();
wxString ConfigPath();
wxString ReadLanguagePreference();
bool SaveConfig(wxFileConfig& config, const wxString& path);
wxString RequestedLanguage();
wxString EffectiveLanguage();
bool SetLanguage(const wxString& code);
wxString ManualPath();
wxString AboutHtml();
}
