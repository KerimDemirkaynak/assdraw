#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import shutil
import subprocess
import urllib.request

DOMAIN = "assdraw"
PO_DIR = "po"

# Hedef diller
TARGET_LANGUAGES = ["tr", "en", "fr", "de", "es", "zh_TW", "vi"]

# Dil menüsünde görünecek diller (Sabit isimlerle, çevrilmeden kalacak)
MENU_LANGUAGES = [
    {"id_num": 6001, "wx_lang": "wxLANGUAGE_ENGLISH", "native": "English"},
    {"id_num": 6002, "wx_lang": "wxLANGUAGE_TURKISH", "native": "Türkçe"},
    {"id_num": 6003, "wx_lang": "wxLANGUAGE_FRENCH", "native": "Français"},
    {"id_num": 6004, "wx_lang": "wxLANGUAGE_GERMAN", "native": "Deutsch"},
    {"id_num": 6005, "wx_lang": "wxLANGUAGE_SPANISH", "native": "Español"},
    {"id_num": 6006, "wx_lang": "wxLANGUAGE_CHINESE_TRADITIONAL", "native": "繁體中文"},
    {"id_num": 6007, "wx_lang": "wxLANGUAGE_VIETNAMESE", "native": "Tiếng Việt"}
]

CALL_FUNCS = [
    "SetTitle", "SetLabel", "SetToolTip", "SetHelpText", "SetStatusText", "SetPage",
    "wxMessageBox", "wxMessageDialog", "Append", "AppendItem", "AppendRadioItem", "AppendCheckItem",
    "Insert", "SetName", "AddUndo", "wxString::Format", "Format",
    "APPENDCOLOURPROP", "APPENDUINTPROP", "APPENDBOOLPROP",
    "AddTool", "AddCheckTool", "AddRadioTool", "AddControl"
]
CTOR_CLASSES = [
    "wxStaticText", "wxButton", "wxCheckBox", "wxRadioButton",
    "wxStaticBox", "wxStaticBoxSizer", "wxMenuItem", "wxMessageDialog",
    "wxPropertyCategory", "wxColourProperty", "wxUIntProperty", "wxBoolProperty"
]

STRLIT = re.compile(
    r'_\(\s*"(?:[^"\\]|\\.)*"\s*\)'
    r'|_T\(\s*"((?:[^"\\]|\\.)*)"\s*\)'
    r'|wxT\(\s*"((?:[^"\\]|\\.)*)"\s*\)'
    r'|"((?:[^"\\]|\\.)*)"',
    re.DOTALL
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
    print(f"[*] Toplam {total_files} dosyada {total_strings} UI metni _() ile güvenle sarıldı.")

def fix_include_once_and_dlgctrl(cache):
    path_inc = "src/include_once.hpp"
    if os.path.exists(path_inc):
        content = cache.read(path_inc)
        content = re.sub(r'const\s+wxString\s+([A-Za-z0-9_]+)\s*=\s*(?:wxT|_T)\(\s*(".*?")\s*\);', r'#define \1 _(\2)', content)

        def repl_array(match):
            inner = match.group(1)
            inner = re.sub(r'(?:_|_T|wxT)\(\s*("[^"]+")\s*\)', r'wxTRANSLATE(\1)', inner)
            return f"wxString ASSDrawTransformDlg::combo_templatesStrings[] = {{{inner}}};"
        
        content = re.sub(r'wxString\s+ASSDrawTransformDlg::combo_templatesStrings\[\]\s*=\s*\{(.*?)\};', repl_array, content, flags=re.DOTALL)
        cache.write(path_inc, content)

    path_dlg = "src/dlgctrl.cpp"
    if os.path.exists(path_dlg):
        content = cache.read(path_dlg)
        old_combo_code = r'combo_templates\s*=\s*new\s*wxComboBox\(\s*this,\s*-1,\s*combo_templatesStrings\[0\],\s*__DPDS__\s*,\s*10,\s*combo_templatesStrings,\s*wxCB_READONLY\s*\);'
        new_combo_code = """wxArrayString translated_templates;
    for (int i = 0; i < combo_templatesCount; i++) translated_templates.Add(wxGetTranslation(combo_templatesStrings[i]));
    combo_templates = new wxComboBox( this, -1, translated_templates[0], __DPDS__ , translated_templates, wxCB_READONLY );"""
        if 'translated_templates' not in content:
            content = re.sub(old_combo_code, new_combo_code, content)
            cache.write(path_dlg, content)

def update_about_and_version(cache):
    # 1. Sürüm numarasını güncelle (assdraw.hpp)
    hpp_path = 'src/assdraw.hpp'
    if os.path.exists(hpp_path):
        hpp_content = cache.read(hpp_path)
        hpp_content = hpp_content.replace('_T("3.0 final")', '_T("3.1")')
        cache.write(hpp_path, hpp_content)

    # 2. About (Hakkında) diyaloğuna metni Regex olmadan, direkt string replace ile hatasız ekle (dlgctrl.cpp)
    dlg_path = 'src/dlgctrl.cpp'
    if os.path.exists(dlg_path):
        dlg_content = cache.read(dlg_path)
        if "Kerim Demirkaynak" not in dlg_content:
            # HTML'in bittiği yeri tam olarak bulup çevrilebilir metni operator+ ile ekliyoruz
            dlg_content = dlg_content.replace(
                '</body></html>")',
                '<br><br>" ) + _("This version was created by Kerim Demirkaynak.") + _T("</body></html>")'
            )
            cache.write(dlg_path, dlg_content)

def get_escaped_utf8(text):
    return "".join(f"\\{b:03o}" for b in text.encode('utf-8'))

def setup_language_support(cache):
    hpp_path = 'src/assdraw.hpp'
    cpp_path = 'src/assdraw.cpp'
    
    hpp_content = cache.read(hpp_path)
    cpp_content = cache.read(cpp_path)

    cpp_content = re.sub(r'\s*wxMenu\*\s*langMenu\s*=\s*new\s*wxMenu;.*?Append\(langMenu,\s*_\("Language"\)\);', '', cpp_content, flags=re.DOTALL)
    cpp_content = re.sub(r'\s*void\s+ASSDrawFrame::OnChangeLanguage\(wxCommandEvent&\s*event\)\s*\{.*?\}', '', cpp_content, flags=re.DOTALL)
    cpp_content = re.sub(r'\s*EVT_MENU_RANGE\(6000,\s*6100,\s*ASSDrawFrame::OnChangeLanguage\)', '', cpp_content)

    if '<wx/intl.h>' not in hpp_content:
        hpp_content = '#include <wx/intl.h>\n#include <wx/stdpaths.h>\n' + hpp_content
        
    if 'wxLocale m_locale;' not in hpp_content:
        hpp_content = re.sub(r'(class\s+ASSDrawApp\s*:\s*public\s+wxApp\s*\{)', r'\1\npublic:\n    wxLocale m_locale;', hpp_content, count=1)
        
    if 'OnChangeLanguage' not in hpp_content:
        hpp_content = re.sub(r'(class\s+ASSDrawFrame\s*:\s*public\s+wxFrame\s*\{)', r'\1\npublic:\n    void OnChangeLanguage(wxCommandEvent& event);', hpp_content, count=1)
        
    cache.write(hpp_path, hpp_content)

    if 'm_locale.Init' not in cpp_content:
        # Sistem dilini zorla okuyan güncellenmiş Init bloğu
        locale_code = """
    wxString configfile = wxFileName(wxStandardPaths::Get().GetUserDataDir(), _T("ASSDraw3.cfg")).GetFullPath();
    wxFileConfig cfg(wxEmptyString, wxEmptyString, configfile);
    long langId = cfg.ReadLong(_T("Language"), wxLANGUAGE_DEFAULT); 
    
    if (langId == wxLANGUAGE_DEFAULT) {
        int sysLang = wxLocale::GetSystemLanguage();
        if (sysLang != wxLANGUAGE_UNKNOWN) {
            langId = sysLang;
        }
    }
    
    m_locale.Init(langId, wxLOCALE_CONV_ENCODING);
    
    wxString exeDir = wxPathOnly(wxStandardPaths::Get().GetExecutablePath());
    m_locale.AddCatalogLookupPathPrefix(exeDir + wxFILE_SEP_PATH + _T("locale"));
    m_locale.AddCatalogLookupPathPrefix(_T("locale"));
    m_locale.AddCatalogLookupPathPrefix(_T("../po")); 
#ifdef __UNIX__
    m_locale.AddCatalogLookupPathPrefix(_T("/usr/share/locale"));
#endif
    m_locale.AddCatalog(_T("assdraw"));
"""
        cpp_content = re.sub(r'(bool\s+ASSDrawApp::OnInit\(\)\s*\{)', r'\1' + locale_code, cpp_content, count=1)

    if 'ASSDrawFrame::OnChangeLanguage' not in cpp_content and 'BEGIN_EVENT_TABLE(ASSDrawFrame' in cpp_content:
        cpp_content = re.sub(r'(BEGIN_EVENT_TABLE\(ASSDrawFrame,\s*wxFrame\))', r'\1\n    EVT_MENU_RANGE(6000, 6100, ASSDrawFrame::OnChangeLanguage)', cpp_content, count=1)

    match = re.search(r'SetMenuBar\(\s*([a-zA-Z0-9_]+)\s*\);', cpp_content)
    if match and 'langMenu->AppendRadioItem' not in cpp_content:
        menubar_var = match.group(1)
        
        menu_items_cpp = ""
        checks_cpp = ""
        for lang in MENU_LANGUAGES:
            menu_id = lang['id_num']
            native = lang['native']
            wx_lang = lang['wx_lang']
            
            menu_items_cpp += f'    langMenu->AppendRadioItem({menu_id}, wxString::FromUTF8("{get_escaped_utf8(native)}"));\n'
            checks_cpp += f'    if (currentLang == {wx_lang}) langMenu->Check({menu_id}, true);\n    else '

        menu_code = f"""
    wxMenu* langMenu = new wxMenu;
    langMenu->AppendRadioItem(6000, _("System Default"));
{menu_items_cpp}
    long currentLang;
    config->Read(_T("Language"), &currentLang, wxLANGUAGE_DEFAULT);
{checks_cpp} langMenu->Check(6000, true);
    
    {menubar_var}->Append(langMenu, _("Language"));
"""
        cpp_content = cpp_content.replace(match.group(0), menu_code + "    " + match.group(0))

    if 'void ASSDrawFrame::OnChangeLanguage' not in cpp_content:
        branches_cpp = ""
        for lang in MENU_LANGUAGES:
            menu_id = lang['id_num']
            wx_lang = lang['wx_lang']
            branches_cpp += f'    else if (event.GetId() == {menu_id}) langId = {wx_lang};\n'

        func_code = f"""
void ASSDrawFrame::OnChangeLanguage(wxCommandEvent& event) {{
    int langId = wxLANGUAGE_DEFAULT;
{branches_cpp.replace('else if', 'if', 1)}

    config->Write(_T("Language"), (long)langId);
    config->Flush();

    wxMessageBox(_("Please restart Assdraw for language changes to take effect."), _("Restart Required"), wxICON_INFORMATION);
}}
"""
        match = re.search(r'void\s+ASSDrawFrame::OnClose\b', cpp_content)
        if match:
            cpp_content = cpp_content[:match.start()] + func_code + "\n\n" + cpp_content[match.start():]
        else:
            cpp_content += "\n" + func_code

    cache.write(cpp_path, cpp_content)

def update_cmake(target_langs):
    path = "CMakeLists.txt"
    if not os.path.exists(path): return
    with open(path, "r", encoding="utf-8") as f: content = f.read()
    
    if "MSGFMT_EXECUTABLE" in content:
        content = re.sub(r'# --- i18n \(gettext\) CMake Entegrasyonu ---.*?endif\(\)\n?', '', content, flags=re.DOTALL)

    linguas_cmake = ";".join(target_langs)
    
    block = f'''
# --- i18n (gettext) CMake Entegrasyonu ---
find_program(MSGFMT_EXECUTABLE msgfmt PATHS "C:/Program Files/Git/usr/bin" "C:/msys64/usr/bin")
set(ASSDRAW_LINGUAS {linguas_cmake})
if(MSGFMT_EXECUTABLE)
    message(STATUS "Harika! msgfmt bulundu: ${{MSGFMT_EXECUTABLE}}")
    set(MO_FILES "")
    foreach(LANG ${{ASSDRAW_LINGUAS}})
        set(PO_FILE "${{CMAKE_SOURCE_DIR}}/po/${{LANG}}.po")
        set(MO_DIR "${{CMAKE_CURRENT_BINARY_DIR}}/locale/${{LANG}}/LC_MESSAGES")
        set(MO_FILE "${{MO_DIR}}/{DOMAIN}.mo")
        add_custom_command(
            OUTPUT ${{MO_FILE}}
            COMMAND ${{CMAKE_COMMAND}} -E make_directory ${{MO_DIR}}
            COMMAND ${{MSGFMT_EXECUTABLE}} -o ${{MO_FILE}} ${{PO_FILE}}
            DEPENDS ${{PO_FILE}}
        )
        list(APPEND MO_FILES ${{MO_FILE}})
    endforeach()
    add_custom_target(translations DEPENDS ${{MO_FILES}})
    
    if(TARGET assdraw)
        add_dependencies(assdraw translations)
        add_custom_command(TARGET assdraw POST_BUILD
            COMMAND ${{CMAKE_COMMAND}} -E make_directory "${{CMAKE_CURRENT_BINARY_DIR}}/locale"
            COMMAND ${{CMAKE_COMMAND}} -E copy_directory
            "${{CMAKE_CURRENT_BINARY_DIR}}/locale"
            "$<TARGET_FILE_DIR:assdraw>/locale"
        )
    endif()
else()
    message(FATAL_ERROR "msgfmt bulunamadı! Dil dosyaları derlenemez.")
endif()
'''
    with open(path, "w", encoding="utf-8") as f: f.write(content.rstrip() + "\n\n" + block.strip() + "\n")

def run_gettext_tools(src_dir, target_langs):
    os.makedirs(PO_DIR, exist_ok=True)
    
    src_files = [os.path.join(root, fn) for root, _, files in os.walk(src_dir) for fn in files if fn.endswith((".cpp", ".hpp", ".h"))]

    pot_path = os.path.join(PO_DIR, f"{DOMAIN}.pot")
    
    if shutil.which("xgettext"):
        subprocess.run(["xgettext", "--keyword=_", "--keyword=wxTRANSLATE", "--from-code=UTF-8", "--package-name", DOMAIN, "-d", DOMAIN, "-o", pot_path] + src_files, check=True)

    for lang in target_langs:
        po_path = os.path.join(PO_DIR, f"{lang}.po")
        
        if not os.path.exists(po_path):
            sys_def = "Sistem Varsayılanı" if lang == "tr" else "System Default"
            lang_str = "Dil" if lang == "tr" else "Language"
            req_str = "Yeniden Başlatma Gerekli" if lang == "tr" else "Restart Required"
            ple_str = "Dil ayarlarının etkinleşmesi için lütfen Assdraw'ı yeniden başlatın." if lang == "tr" else "Please restart Assdraw for language changes to take effect."
            author_str = "Bu sürüm Kerim Demirkaynak tarafından oluşturuldu." if lang == "tr" else "This version was created by Kerim Demirkaynak."
            
            if lang == "en":
                sys_def = lang_str = req_str = ple_str = author_str = ""

            content = f'''msgid ""
msgstr ""
"Project-Id-Version: Assdraw3\\n"
"Language: {lang}\\n"
"MIME-Version: 1.0\\n"
"Content-Type: text/plain; charset=UTF-8\\n"
"Content-Transfer-Encoding: 8bit\\n"

msgid "System Default"
msgstr "{sys_def}"

msgid "Language"
msgstr "{lang_str}"

msgid "Restart Required"
msgstr "{req_str}"

msgid "Please restart Assdraw for language changes to take effect."
msgstr "{ple_str}"

msgid "This version was created by Kerim Demirkaynak."
msgstr "{author_str}"
'''
            with open(po_path, 'w', encoding='utf-8') as f:
                f.write(content)

        if shutil.which("msgmerge") and os.path.exists(pot_path):
            subprocess.run(["msgmerge", "--update", "--backup=off", po_path, pot_path], check=True)

def ensure_chm_file():
    os.makedirs("docs", exist_ok=True)
    chm_path = "docs/ASSDraw3.chm"
    if not os.path.exists(chm_path):
        print(f"[*] Kılavuz indiriliyor: {chm_path}...")
        url = "https://github.com/KerimDemirkaynak/assdraw/raw/refs/heads/master/docs/ASSDraw3.chm"
        try:
            urllib.request.urlretrieve(url, chm_path)
            print("[*] Kılavuz başarıyla indirildi ve docs klasörüne eklendi.")
        except Exception as e:
            print(f"[!] Kılavuz indirilemedi: {e}")

def update_inno_setup():
    ensure_chm_file()
    path = "setup.iss"
    if not os.path.exists(path): return
    with open(path, "r", encoding="utf-8") as f: content = f.read()

    content = content.replace('DestDir: "{app}"', 'DestDir: "{app}\\bin"')
    content = content.replace('Filename: "{app}\\assdraw3.exe"', 'Filename: "{app}\\bin\\assdraw3.exe"')
    content = re.sub(r'\s*skipifsourcedoesntexist', '', content)

    if "locale\\*" not in content:
        insert_line = 'Source: "build-dir\\Release\\locale\\*"; DestDir: "{app}\\bin\\locale"; Flags: ignoreversion recursesubdirs createallsubdirs\n'
        if "[Files]" in content:
            content = content.replace("[Files]\n", "[Files]\n" + insert_line)
            
    if "ASSDraw3.chm" not in content:
        insert_chm = 'Source: "docs\\ASSDraw3.chm"; DestDir: "{app}\\bin"; Flags: ignoreversion\n'
        if "[Files]" in content:
            content = content.replace("[Files]\n", "[Files]\n" + insert_chm)
            
    with open(path, "w", encoding="utf-8") as f: f.write(content)

def main():
    print("=== ASSDraw3 i18n Kesin Fix Başlıyor ===\n")
    
    cache = FileCache()
    wrap_ui_strings("src", cache)
    fix_include_once_and_dlgctrl(cache)
    update_about_and_version(cache)
    setup_language_support(cache)
    cache.flush()
    
    update_cmake(TARGET_LANGUAGES)
    run_gettext_tools("src", TARGET_LANGUAGES)
    update_inno_setup()
    
    print("\n=== İşlem Tamamlandı! ===")

if __name__ == "__main__":
    main()
