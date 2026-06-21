#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import shutil
import subprocess

DOMAIN = "assdraw"
PO_DIR = "po"

CALL_FUNCS = [
    "SetTitle", "SetLabel", "SetToolTip", "SetHelpText", "SetStatusText",
    "wxMessageBox", "Append", "AppendItem", "AppendRadioItem", "AppendCheckItem",
    "Insert", "SetName"
]
CTOR_CLASSES = [
    "wxStaticText", "wxButton", "wxCheckBox", "wxRadioButton",
    "wxStaticBox", "wxStaticBoxSizer", "wxMenuItem"
]

STRLIT = re.compile(
    r'_\(\s*"(?:[^"\\]|\\.)*"\s*\)'
    r'|_T\(\s*"((?:[^"\\]|\\.)*)"\s*\)'
    r'|wxT\(\s*"((?:[^"\\]|\\.)*)"\s*\)'
    r'|"((?:[^"\\]|\\.)*)"'
)

class FileCache:
    def __init__(self):
        self.data = {}
        self.dirty = set()

    def read(self, path):
        if path not in self.data:
            with open(path, "r", encoding="utf-8", errors="ignore") as f:
                self.data[path] = f.read()
        return self.data[path]

    def write(self, path, content):
        self.data[path] = content
        self.dirty.add(path)

    def flush(self):
        for path in self.dirty:
            with open(path, "w", encoding="utf-8") as f:
                f.write(self.data[path])
        n = len(self.dirty)
        self.dirty.clear()
        return n

def find_matching_paren(s, open_idx):
    depth = 0
    i = open_idx
    n = len(s)
    while i < n:
        c = s[i]
        if c == '"':
            i += 1
            while i < n and s[i] != '"':
                if s[i] == "\\": i += 1
                i += 1
        elif c == "(": depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0: return i
        i += 1
    return -1

def looks_translatable(s):
    if not s.strip(): return False
    if not re.search(r"[A-Za-z]", s): return False
    if re.fullmatch(r"%[-+0-9.#lh]*[a-zA-Z]", s): return False
    return True

def wrap_literals(text):
    count = 0
    def repl(m):
        nonlocal count
        if m.group(0).startswith("_("): return m.group(0) 
        inner = m.group(1) or m.group(2) or m.group(3) or ""
        if not looks_translatable(inner): return m.group(0)
        count += 1
        return '_("%s")' % inner
    return STRLIT.sub(repl, text), count

def collect_call_spans(content):
    call_pat = re.compile(r"\b(?:" + "|".join(CALL_FUNCS) + r")\s*\(")
    ctor_pat = re.compile(r"\bnew\s+(?:" + "|".join(CTOR_CLASSES) + r")\s*\(")
    spans = []
    for pat in (call_pat, ctor_pat):
        for m in pat.finditer(content):
            open_idx = m.end() - 1
            close_idx = find_matching_paren(content, open_idx)
            if close_idx != -1:
                spans.append((open_idx, close_idx))
    spans.sort()
    merged = []
    last_end = -1
    for s, e in spans:
        if s > last_end:
            merged.append((s, e))
            last_end = e
    return merged

def wrap_strings_in_content(content):
    spans = collect_call_spans(content)
    if not spans: return content, 0
    pieces = []
    cursor = 0
    total = 0
    for s, e in spans:
        pieces.append(content[cursor:s + 1])
        inner = content[s + 1:e]
        new_inner, n = wrap_literals(inner)
        total += n
        pieces.append(new_inner)
        pieces.append(")")
        cursor = e + 1
    pieces.append(content[cursor:])
    return "".join(pieces), total

def wrap_ui_strings(src_dir, cache):
    total_files, total_strings = 0, 0
    for root, _, files in os.walk(src_dir):
        for fn in files:
            if not fn.endswith((".cpp", ".hpp", ".h")): continue
            path = os.path.join(root, fn)
            content = cache.read(path)
            new_content, n = wrap_strings_in_content(content)
            if n:
                if fn.endswith(".cpp") and "wx/intl.h" not in new_content:
                    new_content = "#include <wx/intl.h>\n" + new_content
                cache.write(path, new_content)
                total_files += 1
                total_strings += n
    print(f"[*] Toplam {total_files} dosyada {total_strings} UI metni güvenle _() ile işaretlendi.")

def setup_language_support(cache):
    hpp_path = 'src/assdraw.hpp'
    cpp_path = 'src/assdraw.cpp'
    
    hpp_content = cache.read(hpp_path)
    cpp_content = cache.read(cpp_path)

    if '<wx/intl.h>' not in hpp_content:
        hpp_content = '#include <wx/intl.h>\n' + hpp_content
        
    if 'wxLocale m_locale;' not in hpp_content:
        hpp_content = re.sub(r'(class\s+ASSDrawApp\s*:\s*public\s+wxApp\s*\{)', r'\1\npublic:\n    wxLocale m_locale;', hpp_content, count=1)
        
    if 'OnChangeLanguage' not in hpp_content:
        hpp_content = re.sub(r'(class\s+ASSDrawFrame\s*:\s*public\s+wxFrame\s*\{)', r'\1\npublic:\n    void OnChangeLanguage(wxCommandEvent& event);', hpp_content, count=1)
        
    cache.write(hpp_path, hpp_content)

    if 'm_locale.Init' not in cpp_content:
        locale_code = """
    wxString configfile = wxFileName(wxStandardPaths::Get().GetUserDataDir(), _T("ASSDraw3.cfg")).GetFullPath();
    wxFileConfig cfg(wxEmptyString, wxEmptyString, configfile);
    long langId = cfg.ReadLong(_T("Language"), wxLANGUAGE_DEFAULT);
    
    m_locale.Init(langId, wxLOCALE_DONT_LOAD_DEFAULT);
    m_locale.AddCatalogLookupPathPrefix(_T("locale"));
    m_locale.AddCatalogLookupPathPrefix(_T("../po")); 
#ifdef __UNIX__
    m_locale.AddCatalogLookupPathPrefix(_T("/usr/share/locale"));
#endif
    m_locale.AddCatalog(_T("assdraw"));
"""
        cpp_content = re.sub(r'(bool\s+ASSDrawApp::OnInit\(\)\s*\{)', r'\1' + locale_code, cpp_content, count=1)

    if 'EVT_MENU_RANGE(6000, 6002' not in cpp_content and 'BEGIN_EVENT_TABLE(ASSDrawFrame' in cpp_content:
        cpp_content = re.sub(r'(BEGIN_EVENT_TABLE\(ASSDrawFrame,\s*wxFrame\))', r'\1\n    EVT_MENU_RANGE(6000, 6002, ASSDrawFrame::OnChangeLanguage)', cpp_content, count=1)

    if 'langMenu->AppendRadioItem' not in cpp_content:
        match = re.search(r'SetMenuBar\(\s*([a-zA-Z0-9_]+)\s*\);', cpp_content)
        if match:
            menubar_var = match.group(1)
            menu_code = f"""
    wxMenu* langMenu = new wxMenu;
    langMenu->AppendRadioItem(6000, _("System Default"));
    langMenu->AppendRadioItem(6001, _("English"));
    langMenu->AppendRadioItem(6002, _("Türkçe"));
    
    long currentLang;
    config->Read(_T("Language"), &currentLang, wxLANGUAGE_DEFAULT);
    if (currentLang == wxLANGUAGE_ENGLISH) langMenu->Check(6001, true);
    else if (currentLang == wxLANGUAGE_TURKISH) langMenu->Check(6002, true);
    else langMenu->Check(6000, true);
    
    {menubar_var}->Append(langMenu, _("Language"));
"""
            cpp_content = cpp_content.replace(match.group(0), menu_code + "\n    " + match.group(0))

    if 'void ASSDrawFrame::OnChangeLanguage' not in cpp_content:
        func_code = """
void ASSDrawFrame::OnChangeLanguage(wxCommandEvent& event) {
    int langId = wxLANGUAGE_DEFAULT;
    if (event.GetId() == 6001) langId = wxLANGUAGE_ENGLISH;
    else if (event.GetId() == 6002) langId = wxLANGUAGE_TURKISH;

    config->Write(_T("Language"), (long)langId);
    config->Flush();

    wxMessageBox(_("Please restart Assdraw for language changes to take effect."), _("Restart Required"), wxICON_INFORMATION);
}
"""
        # Hata Fix: Dosyanın sonundaki süslü parantez karmaşasını atlamak için
        # OnChangeLanguage fonksiyonunu güvenli bir liman olan OnClose fonksiyonunun hemen üstüne enjekte ediyoruz.
        match = re.search(r'void\s+ASSDrawFrame::OnClose\b', cpp_content)
        if match:
            cpp_content = cpp_content[:match.start()] + func_code + "\n\n" + cpp_content[match.start():]
        else:
            # Fallback (asla buraya düşmemesi lazım ama güvenli olsun)
            cpp_content += "\n" + func_code

    cache.write(cpp_path, cpp_content)
    print("[*] wxWidgets Dil Menüsü ve Locale ayarları assdraw.cpp/hpp'ye başarıyla enjekte edildi.")

def update_cmake(target_langs):
    path = "CMakeLists.txt"
    if not os.path.exists(path): return
    with open(path, "r", encoding="utf-8") as f: content = f.read()
    if "MSGFMT_EXECUTABLE" in content: return
    
    linguas_cmake = ";".join(target_langs)
    block = f'''
# --- i18n (gettext) CMake Entegrasyonu ---
find_program(MSGFMT_EXECUTABLE msgfmt)
set(ASSDRAW_LINGUAS {linguas_cmake})
if(MSGFMT_EXECUTABLE)
    set(MO_FILES "")
    foreach(LANG ${{ASSDRAW_LINGUAS}})
        set(PO_FILE "${{CMAKE_SOURCE_DIR}}/po/${{LANG}}.po")
        set(MO_DIR "${{CMAKE_RUNTIME_OUTPUT_DIRECTORY}}/locale/${{LANG}}/LC_MESSAGES")
        set(MO_FILE "${{MO_DIR}}/{DOMAIN}.mo")
        add_custom_command(
            OUTPUT ${{MO_FILE}}
            COMMAND ${{CMAKE_COMMAND}} -E make_directory ${{MO_DIR}}
            COMMAND ${{MSGFMT_EXECUTABLE}} -o ${{MO_FILE}} ${{PO_FILE}}
            DEPENDS ${{PO_FILE}}
        )
        list(APPEND MO_FILES ${{MO_FILE}})
    endforeach()
    add_custom_target(translations ALL DEPENDS ${{MO_FILES}})
else()
    message(WARNING "msgfmt bulunamadı, .po dosyaları derlenmeyecek.")
endif()
'''
    with open(path, "a", encoding="utf-8") as f: f.write(block)

def run_gettext_tools(src_dir, target_langs):
    if shutil.which("xgettext") is None:
        os.makedirs(PO_DIR, exist_ok=True)
        po_path = 'po/tr.po'
        if not os.path.exists(po_path):
            with open(po_path, 'w', encoding='utf-8') as f:
                f.write('msgid ""\nmsgstr ""\n"Project-Id-Version: Assdraw3\\n"\n"Language: tr\\n"\n"MIME-Version: 1.0\\n"\n"Content-Type: text/plain; charset=UTF-8\\n"\n"Content-Transfer-Encoding: 8bit\\n"\n\nmsgid "System Default"\nmsgstr "Sistem Varsayılanı"\n')
        return

    os.makedirs(PO_DIR, exist_ok=True)
    src_files = [os.path.join(root, fn) for root, _, files in os.walk(src_dir) for fn in files if fn.endswith((".cpp", ".hpp", ".h"))]

    pot_path = os.path.join(PO_DIR, f"{DOMAIN}.pot")
    subprocess.run(["xgettext", "--keyword=_", "--keyword=wxTRANSLATE", "--from-code=UTF-8", "--package-name", DOMAIN, "-d", DOMAIN, "-o", pot_path] + src_files, check=True)

    for lang in target_langs:
        po_path = os.path.join(PO_DIR, f"{lang}.po")
        if os.path.exists(po_path):
            subprocess.run(["msgmerge", "--update", "--backup=off", po_path, pot_path], check=True)
        elif shutil.which("msginit"):
            subprocess.run(["msginit", "--no-translator", "-i", pot_path, "-o", po_path, "-l", lang], check=True)
        else:
            shutil.copy(pot_path, po_path)

def main():
    print("=== ASSDraw3 i18n Kesin Çözüm Başlıyor ===\n")
    cache = FileCache()
    wrap_ui_strings("src", cache)
    setup_language_support(cache)
    cache.flush()
    update_cmake(["tr", "en"])
    run_gettext_tools("src", ["tr"])
    print("\n=== İşlem Tamamlandı! ===")

if __name__ == "__main__":
    main()
