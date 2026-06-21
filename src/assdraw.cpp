#include <wx/intl.h>
/*
* Copyright (c) 2007, ai-chan
* All rights reserved.
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*     * Redistributions of source code must retain the above copyright
*       notice, this list of conditions and the following disclaimer.
*     * Redistributions in binary form must reproduce the above copyright
*       notice, this list of conditions and the following disclaimer in the
*       documentation and/or other materials provided with the distribution.
*     * Neither the name of the ASSDraw3 Team nor the
*       names of its contributors may be used to endorse or promote products
*       derived from this software without specific prior written permission.
*
* THIS SOFTWARE IS PROVIDED BY AI-CHAN ``AS IS'' AND ANY
* EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
* WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL AI-CHAN BE LIABLE FOR ANY
* DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
* (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
* LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
* ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
* (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
* SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
///////////////////////////////////////////////////////////////////////////////
// Name:        assdraw.cpp
// Purpose:     ASSDraw main source file
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////


#include "assdraw.hpp"
#include "enums.hpp"
#include "include_once.hpp"
#include <wx/clipbrd.h>
#include <wx/wfstream.h>
#include <wx/filename.h>
#include <wx/dynlib.h>
#include <wx/stdpaths.h>

#if !defined(__WINDOWS__)
#include "xpm/res.h"
#endif

//DEFINE_EVENT_TYPE(wxEVT_SETTINGS_CHANGED)

// initialize wxWidget to accept our App class
IMPLEMENT_APP(ASSDrawApp)

BEGIN_EVENT_TABLE(ASSDrawFrame, wxFrame)
    EVT_MENU_RANGE(6000, 6100, ASSDrawFrame::OnChangeLanguage)
    EVT_TOOL(TB_CLEAR, ASSDrawFrame::OnSelect_Clear)
    EVT_TOOL(TB_PREVIEW, ASSDrawFrame::OnSelect_Preview)
    EVT_TOOL(TB_TRANSFORM, ASSDrawFrame::OnSelect_Transform)
    EVT_TOOL_RANGE(MODE_ARR, MODE_DEL, ASSDrawFrame::OnChoose_Mode)
    EVT_TOOL_RCLICKED(wxID_ANY, ASSDrawFrame::OnToolRClick)
    EVT_COMMAND(wxID_ANY, wxEVT_SETTINGS_CHANGED, ASSDrawFrame::OnSettingsChanged)
    EVT_MENU_RANGE(MENU_TB_ALL, MENU_TB_BGIMG, ASSDrawFrame::OnChoose_TBarRClickMenu)
    EVT_MENU(MENU_CLEAR, ASSDrawFrame::OnSelect_Clear)
    EVT_MENU(MENU_PREVIEW, ASSDrawFrame::OnSelect_Preview)
    EVT_MENU(MENU_TRANSFORM, ASSDrawFrame::OnSelect_Transform)
    EVT_MENU(MENU_LIBRARY, ASSDrawFrame::OnSelect_Library)
    EVT_MENU(MENU_SETTINGS, ASSDrawFrame::OnSelect_Settings)
    EVT_MENU(MENU_RESETPERSPECTIVE, ASSDrawFrame::OnSelect_ResetPerspective)
    EVT_MENU(MENU_HELP, ASSDrawFrame::OnSelect_Help)
    EVT_MENU(wxID_ABOUT, ASSDrawFrame::OnSelect_About)
    EVT_MENU(MENU_UNDO, ASSDrawFrame::OnSelect_Undo)
    EVT_MENU(MENU_REDO, ASSDrawFrame::OnSelect_Redo)
    EVT_MENU(MENU_PASTE, ASSDrawFrame::OnSelect_Paste)
    EVT_MENU(MENU_BGIMG_REMOVE, ASSDrawFrame::OnSelect_RemoveBG)
    EVT_MENU(MENU_BGIMG_ALPHA, ASSDrawFrame::OnSelect_AlphaBG)
    EVT_MENU_RANGE(MODE_ARR, MODE_NUT_BILINEAR, ASSDrawFrame::OnChoose_Mode)
    EVT_MENU_RANGE(MENU_REPOS_TOPLEFT, MENU_REPOS_BOTRIGHT, ASSDrawFrame::OnChoose_Recenter)
    EVT_MENU_RANGE(MENU_REPOS_BGTOPLEFT, MENU_REPOS_BGBOTRIGHT, ASSDrawFrame::OnChoose_RecenterToBG)
	EVT_DPI_CHANGED(ASSDrawFrame::OnDPIChanged)
	EVT_CLOSE(ASSDrawFrame::OnClose)
END_EVENT_TABLE()

// ----------------------------------------------------------------------------
// the application class: ASSDrawApp
// ----------------------------------------------------------------------------

// 'Main program' equivalent: the program execution _T("starts") here
bool ASSDrawApp::OnInit()
{
    wxString configfile = wxFileName(wxStandardPaths::Get().GetUserDataDir(), _T("ASSDraw3.cfg")).GetFullPath();
    wxFileConfig cfg(wxEmptyString, wxEmptyString, configfile);
    long langId = cfg.ReadLong(_T("Language"), wxLANGUAGE_DEFAULT); 
    
    m_locale.Init(langId, wxLOCALE_DONT_LOAD_DEFAULT);
    
    wxString exeDir = wxPathOnly(wxStandardPaths::Get().GetExecutablePath());
    m_locale.AddCatalogLookupPathPrefix(exeDir + wxFILE_SEP_PATH + _T("locale"));
    m_locale.AddCatalogLookupPathPrefix(_T("locale"));
    m_locale.AddCatalogLookupPathPrefix(_T("../po")); 
#ifdef __UNIX__
    m_locale.AddCatalogLookupPathPrefix(_T("/usr/share/locale"));
#endif
    m_locale.AddCatalog(_T("assdraw"));

	SetAppName(TITLE);
    // create the main application window
    ASSDrawFrame * assdrawframe = new ASSDrawFrame( this, TITLE, wxDefaultPosition, wxDefaultSize );
	SetTopWindow(assdrawframe);
    return TRUE;
}



// ----------------------------------------------------------------------------
// main frame: ASSDrawFrame
// ----------------------------------------------------------------------------

// constructor

ASSDrawFrame::ASSDrawFrame( wxApp *app, const wxString& title, const wxPoint& pos, const wxSize& size, long style)
       : wxFrame(NULL, wxID_ANY, title, pos, size, style)
{
	SetSize(FromDIP(wxSize(640, 480)));
	m_app = app;
    m_mgr.SetManagedWindow(this);
#ifndef __UNIX__
    m_mgr.SetFlags(m_mgr.GetFlags() | wxAUI_MGR_ALLOW_ACTIVE_PANE);
#else
	m_mgr.SetFlags(wxAUI_MGR_ALLOW_FLOATING | wxAUI_MGR_ALLOW_ACTIVE_PANE | wxAUI_MGR_RECTANGLE_HINT);
#endif

    // set the frame icon
    SetIcon(wxICON(appico));

   	// Create status bar for the frame
    CreateStatusBar(3);
    int statwidths[] = { FromDIP(64), -1, FromDIP(64) };
    GetStatusBar()->SetStatusWidths(3, statwidths);
    SetStatusBarPane(1);

	InitializeDefaultSettings();

	// load config
	configfile = wxFileName(wxStandardPaths::Get().GetUserDataDir(), _T("ASSDraw3.cfg")).GetFullPath();

	bool firsttime = !::wxFileExists(configfile);
	if (firsttime) {
		wxFileName::Mkdir(wxStandardPaths::Get().GetUserDataDir(), wxS_DIR_DEFAULT, wxPATH_MKDIR_FULL);
		wxFileOutputStream(configfile).Close();
	}
	wxFileInputStream cfgf(configfile);
	config = new wxFileConfig(cfgf);

	// nullify transformdlg
	transformdlg = NULL;

	Maximize(true);
	Show(true); // to get the right client size, must call Show() first

	// config
	LoadSettings();

	// THE CANVAS
    m_canvas = new ASSDrawCanvas( this , this );

	// shapes library
	shapelib = new ASSDrawShapeLibrary(this, this);

	// source text ctrl
	srctxtctrl = new ASSDrawSrcTxtCtrl(this, this);

	// settings
	settingsdlg = new ASSDrawSettingsDialog(this, this);
	settingsdlg->Init();

	SetMenus();
	SetToolBars();
	SetPanes();

	// config

	config->SetPath(_T("info"));
	wxString version;
	config->Read(_T("version"), &version);
	config->SetPath(_T(".."));

	default_perspective = m_mgr.SavePerspective(); // back up default perspective
	config->SetPath(_T("perspective"));
	wxString perspective;
	if (config->Read(_T("perspective"), &perspective) && version == VERSION) m_mgr.LoadPerspective(perspective, false);
	config->SetPath(_T(".."));

	config->SetPath(_T("library"));
	int n = 0;
	config->Read(_T("n"), &n);
	for (int i = 0; i < n; i++)
	{
		wxString libcmds;
		config->Read(wxString::Format(_T("%d"),i), &libcmds);
		shapelib->AddShapePreview(libcmds);
	}
	config->SetPath(_T(".."));

    m_mgr.Update();
	ApplyDpiLayout();
	m_canvas->SetFocus();
	m_canvas->Show();

	wxSize clientsize = m_canvas->GetClientSize();
	m_canvas->ChangeZoomLevelTo(DEFAULT_SCALE, wxPoint(clientsize.x / 2, clientsize.y / 2));
	m_canvas->MoveCanvasOriginTo(clientsize.x / 2, clientsize.y / 2);
	UpdateASSCommandStringToSrcTxtCtrl(m_canvas->GenerateASS());

	UpdateFrameUI();
	ApplySettings();

	if (firsttime)
		_About();
	else if (!behaviors.nosplashscreen)
		_About(3);

	helpcontroller.SetParentWindow(this);
	helpcontroller.Initialize(wxFileName(::wxGetCwd(), _T("ASSDraw3.chm")).GetFullPath());
}

void ASSDrawFrame::SetToolBars()
{
    drawtbar = new wxToolBar(this, wxID_ANY, __DPDS__ , wxTB_FLAT | wxTB_TEXT | wxTB_NODIVIDER | wxTB_HORIZONTAL);
	drawtbar->SetToolBitmapSize(FromDIP(wxSize(16, 16)));
	drawtbar->AddTool(TB_CLEAR, _("Clear"), ASSDRAW_BITMAP_BUNDLE(new_), wxBitmapBundle(), wxITEM_NORMAL, _T(""), TIPS_CLEAR);
    //tbar->AddTool(TB_EDITSRC, _("Source"), ASSDRAW_BITMAP_BUNDLE(src_), wxBitmapBundle(), wxITEM_NORMAL, _T(""), TIPS_EDITSRC);
    drawtbar->AddCheckTool(TB_PREVIEW, _("Preview"), ASSDRAW_BITMAP_BUNDLE(preview_), wxBitmapBundle(), _T(""), TIPS_PREVIEW);
    //drawtbar->AddTool(TB_TRANSFORM, _("Transform"), ASSDRAW_BITMAP_BUNDLE(rot_), wxBitmapBundle(), wxITEM_NORMAL, _T(""), TIPS_TRANSFORM);
	zoomslider = new wxSlider(drawtbar, TB_ZOOMSLIDER, 1000, 100, 5000, __DPDS__ );
	//zoomslider->SetSize(280, zoomslider->GetSize().y);
	zoomslider->Connect(wxEVT_SCROLL_LINEUP, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	zoomslider->Connect(wxEVT_SCROLL_LINEDOWN, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	zoomslider->Connect(wxEVT_SCROLL_PAGEUP, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	zoomslider->Connect(wxEVT_SCROLL_PAGEDOWN, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	zoomslider->Connect(wxEVT_SCROLL_THUMBTRACK, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	zoomslider->Connect(wxEVT_SCROLL_CHANGED, wxScrollEventHandler(ASSDrawFrame::OnZoomSliderChanged), NULL, this);
	drawtbar->AddControl(zoomslider);
    drawtbar->Realize();

    m_mgr.AddPane(drawtbar, wxAuiPaneInfo().Name(_T("drawtbar")).Caption(TBNAME_DRAW).
                  ToolbarPane().Top().Position(0).Dockable(true).LeftDockable(false).RightDockable(false));

    modetbar = new wxToolBar(this, wxID_ANY, __DPDS__ , wxTB_FLAT | wxTB_TEXT | wxTB_NODIVIDER | wxTB_HORIZONTAL);
	modetbar->SetToolBitmapSize(FromDIP(wxSize(16, 16)));
    modetbar->AddRadioTool(MODE_ARR, _("Drag"), ASSDRAW_BITMAP_BUNDLE(arr_), wxBitmapBundle(), _T(""), TIPS_ARR);
    modetbar->AddRadioTool(MODE_M, _("Move"), ASSDRAW_BITMAP_BUNDLE(m_), wxBitmapBundle(), _T(""), TIPS_M);
    //modetbar->AddRadioTool(MODE_N, _("Move*"), ASSDRAW_BITMAP_BUNDLE(n_), wxBitmapBundle(), _T(""), TIPS_N);
    modetbar->AddRadioTool(MODE_L, _("Line"), ASSDRAW_BITMAP_BUNDLE(l_), wxBitmapBundle(), _T(""), TIPS_L);
    modetbar->AddRadioTool(MODE_B, _("Bezier"), ASSDRAW_BITMAP_BUNDLE(b_), wxBitmapBundle(), _T(""), TIPS_B);
    //modetbar->AddRadioTool(MODE_S, _("Spline"), ASSDRAW_BITMAP_BUNDLE(s_), wxBitmapBundle(), _T(""), TIPS_S);
    //modetbar->AddRadioTool(MODE_P, _("Extend"), ASSDRAW_BITMAP_BUNDLE(p_), wxBitmapBundle(), _T(""), TIPS_P);
    //modetbar->AddRadioTool(MODE_C, _("Close"), ASSDRAW_BITMAP_BUNDLE(c_), wxBitmapBundle(), _T(""), TIPS_C);
    modetbar->AddRadioTool(MODE_DEL, _("Delete"), ASSDRAW_BITMAP_BUNDLE(del_), wxBitmapBundle(), _T(""), TIPS_DEL);
    modetbar->AddRadioTool(MODE_SCALEROTATE, _("Scale/Rotate"), ASSDRAW_BITMAP_BUNDLE(sc_rot_), wxBitmapBundle(), _T(""), TIPS_SCALEROTATE);
    modetbar->AddRadioTool(MODE_NUT_BILINEAR, _("Bilinear"), ASSDRAW_BITMAP_BUNDLE(nut_), wxBitmapBundle(), _T(""), TIPS_NUTB);
    //modetbar->AddRadioTool(MODE_NUT_PERSPECTIVE, _("NUT:P"), ASSDRAW_BITMAP_BUNDLE(arr_), wxBitmapBundle(), _T(""), _T(""));
    modetbar->Realize();

    m_mgr.AddPane(modetbar, wxAuiPaneInfo().Name(_T("modetbar")).Caption(TBNAME_MODE).
                  ToolbarPane().Top().Position(1).Dockable(true).LeftDockable(false).RightDockable(false));

    bgimgtbar = new wxToolBar(this, wxID_ANY, __DPDS__ , wxTB_FLAT | wxTB_TEXT | wxTB_NODIVIDER | wxTB_HORIZONTAL);
	bgimgtbar->SetToolBitmapSize(FromDIP(wxSize(24, 15)));
    bgimgtbar->AddCheckTool(DRAG_DWG, _("Pan drawing"), ASSDRAW_BITMAP_BUNDLE(pan_shp), wxBitmapBundle(), _T(""), TIPS_DWG);
    bgimgtbar->AddCheckTool(DRAG_BGIMG, _("Pan background"), ASSDRAW_BITMAP_BUNDLE(pan_bg), wxBitmapBundle(), _T(""), TIPS_BGIMG);
    //bgimgtbar->AddRadioTool(DRAG_BOTH, _("Pan both"), ASSDRAW_BITMAP_BUNDLE(pan_both), wxBitmapBundle(), _T(""), TIPS_BOTH);
    bgimgtbar->Realize();

    m_mgr.AddPane(bgimgtbar, wxAuiPaneInfo().Name(_T("bgimgtbar")).Caption(TBNAME_BGIMG).
                  ToolbarPane().Top().Position(2).Dockable(true).LeftDockable(false).RightDockable(false));

}

void ASSDrawFrame::SetMenus()
{
	drawMenu = new wxMenu;
	drawMenu->Append(MENU_CLEAR, _("&Clear\tCtrl+N"), TIPS_CLEAR);
	//drawMenu->Append(MENU_EDITSRC, _("&Source"), TIPS_EDITSRC);
	drawMenu->Append(MENU_PREVIEW, _("&Preview\tCtrl+P"), TIPS_PREVIEW, wxITEM_CHECK);
	drawMenu->Append(MENU_TRANSFORM, _("&Transform"), TIPS_TRANSFORM);
	drawMenu->Append(MENU_PASTE, _("&Paste\tCtrl+V"), TIPS_PASTE);
	drawMenu->AppendSeparator();
	drawMenu->Append(MENU_UNDO, _("&Undo\tCtrl+Z"), TIPS_UNDO);
	drawMenu->Append(MENU_REDO, _("&Redo\tCtrl+Y"), TIPS_REDO);
	drawMenu->Enable(MENU_UNDO, false);
	drawMenu->Enable(MENU_REDO, false);

	modeMenu = new wxMenu;
	modeMenu->Append(MODE_ARR, _("D&rag\tF1"), TIPS_ARR, wxITEM_RADIO);
	modeMenu->Append(MODE_M, _("Draw &M\tF2"), TIPS_M, wxITEM_RADIO);
	modeMenu->Append(MODE_L, _("Draw &L\tF3"), TIPS_L, wxITEM_RADIO);
	modeMenu->Append(MODE_B, _("Draw &B\tF4"), TIPS_B, wxITEM_RADIO);
	modeMenu->Append(MODE_DEL, _("&Delete\tF5"), TIPS_DEL, wxITEM_RADIO);
	modeMenu->Append(MODE_SCALEROTATE, _("&Scale/Rotate\tF6"), TIPS_NUTB, wxITEM_RADIO);
	modeMenu->Append(MODE_NUT_BILINEAR, _("&Bilinear transformation\tF7"), TIPS_SCALEROTATE, wxITEM_RADIO);

	bgimgMenu = new wxMenu;
	bgimgMenu->Append(DRAG_DWG, _("Pan/Zoom &Drawing\tShift+F1"), TIPS_DWG, wxITEM_CHECK);
	bgimgMenu->Append(DRAG_BGIMG, _("Pan/Zoom Back&ground\tShift+F2"), TIPS_BGIMG, wxITEM_CHECK);
	bgimgMenu->AppendSeparator();
	bgimgMenu->Append(MENU_BGIMG_ALPHA, _("Set background image opacity"), _T(""));
	wxMenu* reposbgMenu = new wxMenu;
	reposbgMenu->Append( MENU_REPOS_BGTOPLEFT, _("Top left\tCtrl+Shift+7") );
	reposbgMenu->Append( MENU_REPOS_BGTOPRIGHT, _("Top right\tCtrl+Shift+9") );
	reposbgMenu->Append( MENU_REPOS_BGCENTER, _("&Center\tCtrl+Shift+5") );
	reposbgMenu->Append( MENU_REPOS_BGBOTLEFT, _("Bottom left\tCtrl+Shift+1") );
	reposbgMenu->Append( MENU_REPOS_BGBOTRIGHT, _("Bottom right\tCtrl+Shift+3") );
	bgimgMenu->Append(MENU_BGIMG_RECENTER, _("Reposition [&0, 0]"), reposbgMenu);
	bgimgMenu->Append(MENU_BGIMG_REMOVE, _("Remove background\tShift+Del"), _T(""));

	wxMenu* reposMenu = new wxMenu;
	reposMenu->Append( MENU_REPOS_TOPLEFT, _("Top left\tCtrl+7") );
	reposMenu->Append( MENU_REPOS_TOPRIGHT, _("Top right\tCtrl+9") );
	reposMenu->Append( MENU_REPOS_CENTER, _("&Center\tCtrl+5") );
	reposMenu->Append( MENU_REPOS_BOTLEFT, _("Bottom left\tCtrl+1") );
	reposMenu->Append( MENU_REPOS_BOTRIGHT, _("Bottom right\tCtrl+3") );

	tbarMenu = new wxMenu;
	tbarMenu->AppendCheckItem(MENU_TB_DRAW, TBNAME_DRAW);
	tbarMenu->AppendCheckItem(MENU_TB_MODE, TBNAME_MODE);
	tbarMenu->AppendCheckItem(MENU_TB_BGIMG, TBNAME_BGIMG);
	tbarMenu->AppendSeparator();
	tbarMenu->Append(MENU_TB_ALL, _("Show all"));
	tbarMenu->Append(MENU_TB_NONE, _("Hide all"));
	tbarMenu->Append(MENU_TB_DOCK, _("Dock all"));
	tbarMenu->Append(MENU_TB_UNDOCK, _("Undock all"));

	viewMenu = new wxMenu;
	viewMenu->Append(MENU_LIBRARY, _("&Library"), TIPS_LIBRARY, wxITEM_CHECK);
	if (settingsdlg)
		viewMenu->Append(MENU_SETTINGS, _("&Settings"), _T(""), wxITEM_CHECK);
	viewMenu->Append(MENU_TBAR, _("&Toolbars"), tbarMenu);
	viewMenu->Append(MENU_RECENTER, _("Reposition [&0, 0]"), reposMenu);
	viewMenu->AppendSeparator();
	viewMenu->Append(MENU_RESETPERSPECTIVE, _("&Reset workspace"));

	wxMenu* helpMenu = new wxMenu;
	helpMenu->Append(MENU_HELP, _("&Manual"));
	helpMenu->Append(wxID_ABOUT, _("&About"));

	wxMenuBar *menuBar = new wxMenuBar();
	menuBar->Append(drawMenu, _("&Canvas"));
	menuBar->Append(modeMenu, _("&Mode"));
	menuBar->Append(bgimgMenu, _("&Background"));
	menuBar->Append(viewMenu, _("&Workspace"));
	menuBar->Append(helpMenu, _("&Help"));


	
    wxMenu* langMenu = new wxMenu;
    langMenu->AppendRadioItem(6000, _("System Default"));
    langMenu->AppendRadioItem(6001, wxT("English"));
    langMenu->AppendRadioItem(6002, wxT("Türkçe"));

    long currentLang;
    config->Read(_T("Language"), &currentLang, wxLANGUAGE_DEFAULT);
    if (currentLang == wxLANGUAGE_ENGLISH) langMenu->Check(6001, true);
    else     if (currentLang == wxLANGUAGE_TURKISH) langMenu->Check(6002, true);
    else  langMenu->Check(6000, true);
    
    menuBar->Append(langMenu, _("Language"));
    SetMenuBar(menuBar);
}

void ASSDrawFrame::SetPanes()
{
	m_mgr.AddPane(shapelib, wxAuiPaneInfo().Name(_T("library")).Caption(_T("Shapes Library")).
                  Right().Layer(2).Position(0).CloseButton(true).BestSize(FromDIP(wxSize(120, 480))).MinSize(FromDIP(wxSize(100, 200))));

	m_mgr.AddPane(m_canvas, wxAuiPaneInfo().Name(_T("canvas")).CenterPane());

	m_mgr.AddPane(srctxtctrl, wxAuiPaneInfo().Name(_T("commands")).Caption(_T("Drawing commands")).
                  Bottom().Layer(1).CloseButton(false).BestSize(FromDIP(wxSize(320, 48))));

	if (settingsdlg)
		m_mgr.AddPane(settingsdlg, wxAuiPaneInfo().Name(_T("settings")).Caption(_T("Settings")).
                  Right().Layer(3).Position(0).CloseButton(true).BestSize(FromDIP(wxSize(240, 480))).MinSize(FromDIP(wxSize(200, 200))).Show(false));
}

void ASSDrawFrame::ApplyDpiLayout()
{
	if (GetStatusBar())
	{
		int statwidths[] = { FromDIP(64), -1, FromDIP(64) };
		GetStatusBar()->SetStatusWidths(3, statwidths);
	}

	if (drawtbar)
		drawtbar->Realize();
	if (modetbar)
		modetbar->Realize();
	if (bgimgtbar)
		bgimgtbar->Realize();

	if (shapelib)
		shapelib->UpdateDpiLayout();

	auto update_toolbar_pane = [this](wxWindow *window)
	{
		if (window == NULL)
			return;

		wxAuiPaneInfo& pane = m_mgr.GetPane(window);
		if (pane.IsOk())
			pane.BestSize(window->GetBestSize());
	};

	update_toolbar_pane(drawtbar);
	update_toolbar_pane(modetbar);
	update_toolbar_pane(bgimgtbar);

	wxAuiPaneInfo& library_pane = m_mgr.GetPane(shapelib);
	if (library_pane.IsOk())
		library_pane.BestSize(FromDIP(wxSize(120, 480))).MinSize(FromDIP(wxSize(100, 200)));

	wxAuiPaneInfo& commands_pane = m_mgr.GetPane(srctxtctrl);
	if (commands_pane.IsOk())
		commands_pane.BestSize(FromDIP(wxSize(320, 48)));

	if (settingsdlg)
	{
		wxAuiPaneInfo& settings_pane = m_mgr.GetPane(settingsdlg);
		if (settings_pane.IsOk())
			settings_pane.BestSize(FromDIP(wxSize(240, 480))).MinSize(FromDIP(wxSize(200, 200)));
	}

	m_mgr.Update();
	Layout();
	Refresh();
}

ASSDrawFrame::~ASSDrawFrame()
{
	config->SetPath(_T("info"));
	config->Write(_T("assdraw3.exe"), wxStandardPaths::Get().GetExecutablePath());
	config->Write(_T("version"), VERSION);
	config->SetPath(_T(".."));

	SaveSettings();

	config->SetPath(_T("perspective"));
	config->Write(_T("perspective"), m_mgr.SavePerspective());
	config->SetPath(_T(".."));

	config->DeleteGroup(_T("library"));
	config->SetPath(_T("library"));
	typedef std::vector< ASSDrawShapePreview *> PrevVec;
	PrevVec shapes = shapelib->GetShapePreviews();
	int n = shapes.size();
	config->Write(_T("n"), n);
	for (int i = 0; i < n; i++)
		config->Write(wxString::Format(_T("%d"),i), shapes[i]->GenerateASS());
	config->SetPath(_T(".."));

	wxFileOutputStream cfgf(configfile);
	config->Save(cfgf);
	delete config;

	if (settingsdlg) settingsdlg->Destroy(); // we need this since wxPropertyGrid must be Clear()ed before deleting

	m_mgr.UnInit();
}

void ASSDrawFrame::_Clear()
{
	wxMessageDialog msgb(this, _T("Clear the canvas and create a new drawing?"),
                  _T("Confirmation"), wxOK | wxCANCEL | wxICON_QUESTION );
	if (msgb.ShowModal() == wxID_OK)
	{
		m_canvas->RefreshUndocmds();
		m_canvas->AddUndo(_("Clear canvas"));
		m_canvas->ResetEngine(true);
	    wxSize siz = m_canvas->GetClientSize();
		m_canvas->ChangeZoomLevelTo(DEFAULT_SCALE, wxPoint(siz.x / 2, siz.y / 2));
		m_canvas->MoveCanvasOriginTo(siz.x / 2, siz.y / 2);
		UpdateUndoRedoMenu();
		m_canvas->RefreshDisplay();
	}
}

void ASSDrawFrame::_Preview()
{
	m_canvas->SetPreviewMode( !m_canvas->IsPreviewMode() );
	UpdateFrameUI();
	m_canvas->RefreshDisplay();
}

void ASSDrawFrame::_ToggleLibrary()
{
	m_mgr.GetPane(shapelib).Show(!m_mgr.GetPane(shapelib).IsShown());
	m_mgr.Update();
	UpdateFrameUI();
}

void ASSDrawFrame::_ToggleSettings()
{
	if (settingsdlg == NULL) return;
	m_mgr.GetPane(settingsdlg).Show(!m_mgr.GetPane(settingsdlg).IsShown());
	m_mgr.Update();
	UpdateFrameUI();
}

void ASSDrawFrame::_ResetPerspective()
{
	m_mgr.LoadPerspective(default_perspective, false);
	ApplyDpiLayout();
	UpdateFrameUI();
	DRAGMODE bck = m_canvas->GetDragMode();
	if (m_canvas->HasBackgroundImage()) m_canvas->SetDragMode(DRAGMODE(true, true));
	wxSize clientsize = m_canvas->GetClientSize();
	m_canvas->ChangeZoomLevelTo(DEFAULT_SCALE, wxPoint(clientsize.x / 2, clientsize.y / 2));
	m_canvas->MoveCanvasOriginTo(clientsize.x / 2, clientsize.y / 2);
	m_canvas->SetDragMode(bck);
	UpdateFrameUI();
}

void ASSDrawFrame::_Transform()
{
	if (transformdlg == NULL)
		transformdlg = new ASSDrawTransformDlg( this );

	if (transformdlg->ShowModal() == wxID_OK)
	{
		m_canvas->Transform(
			transformdlg->xformvals.f1,
			transformdlg->xformvals.f2,
			transformdlg->xformvals.f3,
			transformdlg->xformvals.f4,
			transformdlg->xformvals.f5,
			transformdlg->xformvals.f6,
			transformdlg->xformvals.f7,
			transformdlg->xformvals.f8 );
		m_canvas->AddUndo(_("Transform"));
		m_canvas->RefreshDisplay();
		UpdateUndoRedoMenu();
	}
}

void ASSDrawFrame::_Paste()
{
	if (wxTheClipboard->Open())
	{
		if (wxTheClipboard->IsSupported( wxDF_BITMAP ))
		{
			wxBitmapDataObject data;
			wxTheClipboard->GetData( data );
			m_canvas->SetBackgroundImage( data.GetBitmap().ConvertToImage() );
			//m_canvas->AskUserForBackgroundAlpha();
		}
		else if (wxTheClipboard->IsSupported( wxDF_TEXT ))
		{
			wxTextDataObject data;
			wxTheClipboard->GetData( data );
			UpdateASSCommandStringToSrcTxtCtrl( data.GetText() );
		}
		wxTheClipboard->Close();
	}
}

void ASSDrawFrame::OnChoose_Recenter(wxCommandEvent& event)
{
	int x = 0, y = 0;
	wxSize f = m_canvas->GetClientSize();

	switch (event.GetId())
	{
	case MENU_REPOS_TOPLEFT: x = 0, y = 0; break;
	case MENU_REPOS_TOPRIGHT: x = f.x, y = 0; break;
	case MENU_REPOS_CENTER: x = f.x / 2, y = f.y / 2; break;
	case MENU_REPOS_BOTLEFT: x = 0, y = f.y; break;
	case MENU_REPOS_BOTRIGHT: x = f.x, y = f.y; break;
	}

	m_canvas->MoveCanvasOriginTo(x, y);
	m_canvas->RefreshDisplay();
}

void ASSDrawFrame::OnChoose_RecenterToBG(wxCommandEvent& event)
{
	unsigned w, h;
	wxRealPoint disp;
	double scale;
	if (m_canvas->GetBackgroundInfo(w, h, disp, scale))
	{
		int x = 0, y = 0;
		int lx = (int)disp.x, ty = (int)disp.y;
		int rx = lx + (int)(w * scale);
		int by = ty + (int)(h * scale);

		switch (event.GetId())
		{
		case MENU_REPOS_BGTOPLEFT: x = lx, y = ty; break;
		case MENU_REPOS_BGTOPRIGHT: x = rx, y = ty; break;
		case MENU_REPOS_BGCENTER: x = (rx + lx) / 2, y = (by + ty) / 2; break;
		case MENU_REPOS_BGBOTLEFT: x = lx, y = by; break;
		case MENU_REPOS_BGBOTRIGHT: x = rx, y = by; break;
		}

		m_canvas->MoveCanvasDrawing(x - m_canvas->GetOriginX(), y - m_canvas->GetOriginY());
		m_canvas->RefreshDisplay();
	}
}

void ASSDrawFrame::_Help()
{
	helpcontroller.DisplayContents();
}

void ASSDrawFrame::_About(unsigned timeout)
{
	ASSDrawAboutDlg *aboutdlg = new ASSDrawAboutDlg( this, timeout );
	aboutdlg->ShowModal();
	aboutdlg->Destroy();
}

void ASSDrawFrame::OnChoose_Mode(wxCommandEvent& event)
{
	m_canvas->SetDrawMode( (MODE) event.GetId() );
	UpdateFrameUI();
}

void ASSDrawFrame::OnChoose_DragMode(wxCommandEvent& event)
{
	DRAGMODE dm = m_canvas->GetDragMode();
	switch (event.GetId())
	{
		case DRAG_DWG: dm.drawing = !dm.drawing; break;
		case DRAG_BGIMG: dm.bgimg = !dm.bgimg; break;
	}
	m_canvas->SetDragMode( dm );
	UpdateFrameUI();
}

void ASSDrawFrame::OnZoomSliderChanged(wxScrollEvent &event)
{
	double zoom = (double) event.GetPosition() / 100.0;
	m_canvas->ChangeZoomLevelTo(zoom, wxPoint((int) m_canvas->GetOriginX(), (int) m_canvas->GetOriginY()));
}

void ASSDrawFrame::OnToolRClick(wxCommandEvent& event)
{
	int id = event.GetId();
	if (drawtbar->FindById(id) != NULL
		|| modetbar->FindById(id) != NULL
		|| bgimgtbar->FindById(id) != NULL)
	{
		PopupMenu(tbarMenu);
	}
}

void ASSDrawFrame::OnChoose_TBarRClickMenu(wxCommandEvent& event)
{
	int id = event.GetId();
	wxToolBar* tbar[3] = { drawtbar, modetbar, bgimgtbar };
	bool tb[3] = { false, false, false };
	bool show[2] = { false, true };
	bool dock[2] = { false, true };
	switch (id)
	{
	case MENU_TB_ALL:
		tb[0] = true, tb[1] = true, tb[2] = true;
		show[0] = true, show[1] = true;
		break;
	case MENU_TB_NONE:
		tb[0] = true, tb[1] = true, tb[2] = true;
		show[0] = true, show[1] = false;
		break;
	case MENU_TB_DOCK:
		tb[0] = true, tb[1] = true, tb[2] = true;
		dock[0] = true, dock[1] = true;
		break;
	case MENU_TB_UNDOCK:
		tb[0] = true, tb[1] = true, tb[2] = true;
		dock[0] = true, dock[1] = false;
		break;
	case MENU_TB_DRAW:
		tb[0] = true;
		show[0] = true, show[1] = !m_mgr.GetPane(tbar[0]).IsShown();
		break;
	case MENU_TB_MODE:
		tb[1] = true;
		show[0] = true, show[1] = !m_mgr.GetPane(tbar[1]).IsShown();
		break;
	case MENU_TB_BGIMG:
		tb[2] = true;
		show[0] = true, show[1] = !m_mgr.GetPane(tbar[2]).IsShown();
		break;
	}
	for (int i = 0; i < 3; i++)
	{
		if (tb[i])
		{
			if (show[0])
				m_mgr.GetPane(tbar[i]).Show(show[1]);
			if (dock[0])
				if (dock[1])
					m_mgr.GetPane(tbar[i]).Dock();
				else
					m_mgr.GetPane(tbar[i]).Float();
		}
	}
	m_mgr.Update();
	UpdateFrameUI();
}

void ASSDrawFrame::UpdateASSCommandStringFromSrcTxtCtrl(wxString cmds)
{
	m_canvas->ParseASS(cmds, true);
	m_canvas->RefreshDisplay();
}

void ASSDrawFrame::UpdateASSCommandStringToSrcTxtCtrl(wxString cmd)
{
	if (behaviors.capitalizecmds)
		cmd.UpperCase();
	else
		cmd.LowerCase();
	srctxtctrl->ChangeValue(cmd);
	//srctxtctrl->AppendText(cmd);
}

void ASSDrawFrame::UndoOrRedo(bool isundo)
{
	if (isundo)
		m_canvas->Undo();
	else
		m_canvas->Redo();
	UpdateUndoRedoMenu();
	UpdateFrameUI();
}

void ASSDrawFrame::UpdateUndoRedoMenu()
{
	wxString nextUndo = m_canvas->GetTopUndo();
	if (nextUndo.IsSameAs(_T("")))
	{
		drawMenu->SetLabel(MENU_UNDO, _("Undo\tCtrl+Z"));
		drawMenu->Enable(MENU_UNDO, false);
	}
	else
	{
		drawMenu->SetLabel(MENU_UNDO, wxString::Format(_("Undo: %s\tCtrl+Z"), nextUndo.c_str()));
		drawMenu->Enable(MENU_UNDO, true);
	}
	wxString nextRedo = m_canvas->GetTopRedo();
	if (nextRedo.IsSameAs(_T("")))
	{
		drawMenu->SetLabel(MENU_REDO, _("Redo\tCtrl+Y"));
		drawMenu->Enable(MENU_REDO, false);
	}
	else
	{
		drawMenu->SetLabel(MENU_REDO, wxString::Format(_("Redo: %s\tCtrl+Y"), nextRedo.c_str()));
		drawMenu->Enable(MENU_REDO, true);
	}
}

void ASSDrawFrame::UpdateFrameUI(unsigned level)
{
	bool hasbg = m_canvas->HasBackgroundImage();
	int zoom = (int) ((m_canvas->GetScale() * 100.0)+0.5);
	switch (level)
	{
	case 0: // all
		drawtbar->ToggleTool(TB_PREVIEW, m_canvas->IsPreviewMode());
		modetbar->ToggleTool(m_canvas->GetDrawMode(), true);
		drawMenu->Check(MENU_PREVIEW, m_canvas->IsPreviewMode());
		modeMenu->Check(m_canvas->GetDrawMode(), true);
	case 2: // bgimg & toolbars
		bgimgtbar->ToggleTool(DRAG_DWG, m_canvas->GetDragMode().drawing);
		bgimgtbar->ToggleTool(DRAG_BGIMG, m_canvas->GetDragMode().bgimg);
		bgimgtbar->EnableTool(DRAG_BGIMG, hasbg);
		m_mgr.Update();
		viewMenu->Check(MENU_LIBRARY, m_mgr.GetPane(shapelib).IsShown());
		if (settingsdlg)
			viewMenu->Check(MENU_SETTINGS, m_mgr.GetPane(settingsdlg).IsShown());
		bgimgMenu->Check(DRAG_DWG, m_canvas->GetDragMode().drawing);
		bgimgMenu->Check(DRAG_BGIMG, m_canvas->GetDragMode().bgimg);
		bgimgMenu->Enable(DRAG_BGIMG, hasbg);
		bgimgMenu->Enable(MENU_BGIMG_ALPHA, hasbg);
		bgimgMenu->Enable(MENU_BGIMG_RECENTER, hasbg);
		bgimgMenu->Enable(MENU_BGIMG_REMOVE, hasbg);
		tbarMenu->Check(MENU_TB_DRAW, m_mgr.GetPane(drawtbar).IsShown());
		tbarMenu->Check(MENU_TB_MODE, m_mgr.GetPane(modetbar).IsShown());
		tbarMenu->Check(MENU_TB_BGIMG, m_mgr.GetPane(bgimgtbar).IsShown());
	case 3:	// zoom slider
		zoomslider->SetValue(zoom);
		SetStatusText( wxString::Format(_("%d%%"), zoom), 2 );
		zoomslider->Enable(m_canvas->GetDragMode().drawing && m_canvas->CanZoom());
	}
}

void ASSDrawFrame::OnDPIChanged(wxDPIChangedEvent& event)
{
	CallAfter(&ASSDrawFrame::ApplyDpiLayout);
	event.Skip();
}


void ASSDrawFrame::OnChangeLanguage(wxCommandEvent& event) {
    int langId = wxLANGUAGE_DEFAULT;
    if (event.GetId() == 6001) langId = wxLANGUAGE_ENGLISH;
    else if (event.GetId() == 6002) langId = wxLANGUAGE_TURKISH;


    config->Write(_T("Language"), (long)langId);
    config->Flush();

    wxMessageBox(_("Please restart Assdraw for language changes to take effect."), _("Restart Required"), wxICON_INFORMATION);
}


void ASSDrawFrame::OnClose(wxCloseEvent &event)
{
	if (event.CanVeto() && behaviors.confirmquit)
	{
		if (wxMessageDialog(this, _("Do you want to close ASSDraw3 now?"), _("Confirmation"), wxOK | wxCANCEL).ShowModal() == wxID_OK)
			Destroy();
		else
			event.Veto();
	}
	else
		Destroy();
}
