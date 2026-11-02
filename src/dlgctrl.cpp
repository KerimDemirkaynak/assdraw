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
// Name:        dlgctrl.cpp
// Purpose:     custom dialogs and controls
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////

#include "assdraw.hpp"

#include "xpm/res.h"

static wxBitmapBundle LoadAboutBannerBitmapBundle()
{
#if defined(__WXMSW__)
	wxBitmapBundle bundle = wxBitmapBundle::FromResources(_T("ASSDRAW3_"));
	if (bundle.IsOk())
		return bundle;
#endif

	return wxBitmapBundle::FromBitmap(wxBitmap(assdraw3__xpm));
}

BEGIN_EVENT_TABLE(ASSDrawSrcTxtCtrl, wxTextCtrl)
	EVT_CHAR(ASSDrawSrcTxtCtrl::CustomOnChar)
	EVT_TEXT(wxID_ANY, ASSDrawSrcTxtCtrl::CustomOnText)
END_EVENT_TABLE()

BEGIN_EVENT_TABLE(ASSDrawTransformDlg, wxDialog)
	EVT_COMBOBOX(-1, ASSDrawTransformDlg::OnTemplatesCombo)
END_EVENT_TABLE()

//BEGIN_EVENT_TABLE(ASSDrawCanvasRecenterButton, wxWindow)
//END_EVENT_TABLE()

// ----------------------------------------------------------------------------
// ASSDrawSrcTxtCtrl
// ----------------------------------------------------------------------------

ASSDrawSrcTxtCtrl::ASSDrawSrcTxtCtrl(wxWindow *parent, ASSDrawFrame *frame)
	: wxTextCtrl(parent, wxID_ANY, _T(""), __DPDS__ , wxTE_MULTILINE )
{
	m_frame = frame;
}

void ASSDrawSrcTxtCtrl::CustomOnChar(wxKeyEvent &event)
{
	switch (event.GetKeyCode())
	{
	case WXK_RETURN:
		m_frame->UpdateASSCommandStringFromSrcTxtCtrl(GetValue());
		return; // Enter applies; it must not also insert a line break.
	case WXK_TAB:
		break; //do nothing
	default:
		//m_frame->SetTitle(wxString::Format(_("Key: %d"), event.GetKeyCode()));
		event.Skip(true);
	}

	//SetBackgroundColour(IsModified()? wxColour(0xFF, 0xFF, 0x99):*wxWHITE);
}

void ASSDrawSrcTxtCtrl::CustomOnText(wxCommandEvent &event)
{
	//SetBackgroundColour(IsModified()? wxColour(0xFF, 0xFF, 0x99):*wxWHITE);
}

// ----------------------------------------------------------------------------
// ASSDrawTransformDlg
// ----------------------------------------------------------------------------

ASSDrawTransformDlg::ASSDrawTransformDlg(ASSDrawFrame* parent)
 : wxDialog(parent, -1, wxString(_("Transform")))
{
	m_frame = parent;
	Bind(wxEVT_DPI_CHANGED, &ASSDrawTransformDlg::OnDPIChanged, this);

    wxBoxSizer* sizer_main = new wxBoxSizer(wxVERTICAL);
    this->SetSizer(sizer_main);

    wxBoxSizer* sizer_templates = new wxBoxSizer(wxHORIZONTAL);
	const int border = FromDIP(5);

	sizer_main->Add(sizer_templates, 0, wxGROW|wxLEFT, border);

    sizer_templates->Add(new wxStaticText( this, -1, _("Templates"), __DPDS__ , 0 ),
						0, wxALIGN_CENTER_VERTICAL|wxALL, border);

    wxArrayString templates;
    for (int i = 0; i < combo_templatesCount; ++i)
        templates.Add(wxGetTranslation(wxString::FromUTF8(combo_templatesStrings[i])));
    combo_templates = new wxComboBox(this, -1, templates[0], __DPDS__, templates, wxCB_READONLY);
    sizer_templates->Add(combo_templates, 0, wxALIGN_CENTER_VERTICAL|wxALL, border);

    wxFlexGridSizer* sizer_fields = new wxFlexGridSizer(4, 4, 0, 0);
    sizer_main->Add(sizer_fields, 0, wxALIGN_CENTER_HORIZONTAL|wxLEFT, border);

	int flag_txtctrl = wxALIGN_CENTER_HORIZONTAL|wxALIGN_CENTER_VERTICAL|wxALL;
	int flag_statictxt = wxALIGN_RIGHT|wxALIGN_CENTER_VERTICAL|wxALL;

	sizer_fields->Add(new wxStaticText( this, -1, _T("m11"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_m11 = new wxTextCtrl( this, -1, _T("1.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_m11, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("m12"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_m12 = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_m12, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("m21"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_m21 = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_m21, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("m22"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_m22 = new wxTextCtrl( this, -1, _T("1.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_m22, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("mx"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_mx = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_mx, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("my"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_my = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_my, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("nx"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_nx = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_nx, 0, flag_txtctrl, border);

    sizer_fields->Add(new wxStaticText( this, -1, _T("ny"), __DPDS__ , 0 ),
						0, flag_statictxt, border);

    txtctrl_ny = new wxTextCtrl( this, -1, _T("0.0"), __DPDS__ , wxTE_RIGHT );
    sizer_fields->Add(txtctrl_ny, 0, flag_txtctrl, border);

    auto* formula = new wxStaticText(this, wxID_ANY,
        "x' = m11(x - mx) + m12(y - my) + nx\ny' = m21(x - mx) + m22(y - my) + ny");
    sizer_main->Add(formula, 0, wxALIGN_CENTER_HORIZONTAL | wxALL, border);

    wxStdDialogButtonSizer* sizer_stdbutt = new wxStdDialogButtonSizer;

    sizer_main->Add(sizer_stdbutt, 0, wxALIGN_CENTER_HORIZONTAL|wxALL, border);
    wxButton* button_ok = new wxButton( this, wxID_OK, _("&OK"), __DPDS__ , 0 );
    sizer_stdbutt->AddButton(button_ok);

    wxButton* button_cancel = new wxButton( this, wxID_CANCEL, _("&Cancel"), __DPDS__ , 0 );
    sizer_stdbutt->AddButton(button_cancel);

    sizer_stdbutt->Realize();

	sizer_main->Fit(this);

}

void ASSDrawTransformDlg::UpdateDpiLayout()
{

	Layout();
	Fit();
}

void ASSDrawTransformDlg::OnDPIChanged(wxDPIChangedEvent &event)
{
	UpdateDpiLayout();
	event.Skip();
}

void ASSDrawTransformDlg::OnTemplatesCombo(wxCommandEvent &event)
{
	// The translated label is presentation only; the index selects the matrix.
	const int pos = combo_templates->GetSelection();
	if (pos < 0 || pos >= combo_templatesCount)
		return;

	txtctrl_m11->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f1) );
	txtctrl_m12->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f2) );
	txtctrl_m21->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f3) );
	txtctrl_m22->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f4) );
	txtctrl_mx->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f5) );
	txtctrl_my->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f6) );
	txtctrl_nx->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f7) );
	txtctrl_ny->SetValue( wxString::Format(_T("%.1f"), combo_templatesValues[pos].f8) );
}

void ASSDrawTransformDlg::EndModal(int retCode)
{
	if (retCode != wxID_OK)
	{
		wxDialog::EndModal(retCode);
		return;
	}

	bool ok = true;

	ok = ok && txtctrl_m11->GetValue().ToDouble( &xformvals.f1 );
	ok = ok && txtctrl_m12->GetValue().ToDouble( &xformvals.f2 );
	ok = ok && txtctrl_m21->GetValue().ToDouble( &xformvals.f3 );
	ok = ok && txtctrl_m22->GetValue().ToDouble( &xformvals.f4 );
	ok = ok && txtctrl_mx->GetValue().ToDouble( &xformvals.f5 );
	ok = ok && txtctrl_my->GetValue().ToDouble( &xformvals.f6 );
	ok = ok && txtctrl_nx->GetValue().ToDouble( &xformvals.f7 );
	ok = ok && txtctrl_ny->GetValue().ToDouble( &xformvals.f8 );

	if (ok)
		wxDialog::EndModal(wxID_OK);
	else
	    wxMessageBox(_("One or more values entered are not real numbers.\nPlease fix."), _("Value error"), wxOK | wxICON_INFORMATION, m_frame);

}


ASSDrawAboutDlg::ASSDrawAboutDlg(ASSDrawFrame *parent, unsigned timeout)
	: wxDialog(parent, wxID_ANY, wxString(TITLE), __DPDS__ , wxSIMPLE_BORDER), time_out(timeout)
{
	SetBackgroundColour(AssdrawTheme::Surface());
	html_logical_size = wxSize(396, 200);
	htmlwin = new wxHtmlWindow(this, wxID_ANY, wxDefaultPosition, FromDIP(html_logical_size), wxHW_DEFAULT_STYLE | wxSIMPLE_BORDER);
	htmlwin->SetPage(AssdrawI18n::AboutHtml());
	htmlwin->Connect(wxEVT_COMMAND_HTML_LINK_CLICKED, wxHtmlLinkEventHandler(ASSDrawAboutDlg::OnURL), NULL, this);

	wxFlexGridSizer *sizer = new wxFlexGridSizer(1);
	sizer->AddGrowableCol(0);
	//sizer->AddGrowableRow(1);

	sizer->Add(new BigStaticBitmapCtrl(this, LoadAboutBannerBitmapBundle(), AssdrawTheme::Surface(), this), 1, wxEXPAND);
	sizer->Add(htmlwin, 1, wxLEFT | wxRIGHT, FromDIP(2));
	sizer->Add(new wxStaticText(this, wxID_ANY, wxString::Format(_("Version: %s"), VERSION)), 1, wxEXPAND | wxALL, FromDIP(2));
	sizer->Add(new wxButton(this, wxID_OK, _("OK")), 0, wxALIGN_CENTER | wxBOTTOM, FromDIP(10));
	SetSizer(sizer);
	sizer->Layout();
	sizer->Fit(this);

	CentreOnScreen();
	//if (CanSetTransparent()) SetTransparent(0xCC);

	timer.SetOwner(this);
	Bind(wxEVT_DPI_CHANGED, &ASSDrawAboutDlg::OnDPIChanged, this);
	Connect(wxEVT_TIMER, wxTimerEventHandler(ASSDrawAboutDlg::OnTimeout));
	Connect(wxEVT_ENTER_WINDOW, wxMouseEventHandler(ASSDrawAboutDlg::OnMouseEnterWindow));
}

ASSDrawAboutDlg::~ASSDrawAboutDlg()
{
	timer.Stop();
}

int ASSDrawAboutDlg::ShowModal()
{
	if (time_out > 0)
		timer.Start(time_out * 1000, true);
	return wxDialog::ShowModal();
}

void ASSDrawAboutDlg::OnURL(wxHtmlLinkEvent &event)
{
	wxString URL(event.GetLinkInfo().GetHref());
		if (URL.StartsWith(_T("http://")) || URL.StartsWith(_T("https://")))
		::wxLaunchDefaultBrowser(URL);
	else
		event.Skip(true);
}

void ASSDrawAboutDlg::OnTimeout(wxTimerEvent& event)
{
	if (IsShown())
		EndModal(wxID_OK);
}

void ASSDrawAboutDlg::OnMouseEnterWindow(wxMouseEvent& event)
{
	// if mouse enters this dialog, stop the timout timer
	// and dialog will only close through user input
	timer.Stop();
}

void ASSDrawAboutDlg::UpdateDpiLayout()
{
	if (htmlwin)
	{
		const wxSize html_size = FromDIP(html_logical_size);
		htmlwin->SetMinSize(html_size);
		htmlwin->SetSize(html_size);
	}

	Layout();
	Fit();
}

void ASSDrawAboutDlg::OnDPIChanged(wxDPIChangedEvent& event)
{
	UpdateDpiLayout();
	event.Skip();
}

BEGIN_EVENT_TABLE(BigStaticBitmapCtrl, wxPanel)
	EVT_PAINT(BigStaticBitmapCtrl::OnPaint)
	EVT_DPI_CHANGED(BigStaticBitmapCtrl::OnDPIChanged)
    EVT_MOTION (BigStaticBitmapCtrl::OnMouseMove)
    EVT_LEFT_UP(BigStaticBitmapCtrl::OnMouseLeftUp)
    EVT_LEFT_DOWN(BigStaticBitmapCtrl::OnMouseLeftDown)
END_EVENT_TABLE()

BigStaticBitmapCtrl::BigStaticBitmapCtrl(wxWindow *parent, wxBitmapBundle bmap, wxColour bgcol, wxWindow *todrag)
	: wxPanel(parent, wxID_ANY)
{
	bitmap = bmap;
	bitmap_logical_size = bitmap.IsOk() ? bitmap.GetDefaultSize() : wxSize(1, 1);
	SetBackgroundColour(bgcol);
	window_to_drag = todrag;
	SetBackgroundStyle(wxBG_STYLE_PAINT);

	UpdateBitmapSize();
	Refresh();
}

BigStaticBitmapCtrl::~BigStaticBitmapCtrl()
{
}

void BigStaticBitmapCtrl::OnPaint(wxPaintEvent& event)
{
	wxPaintDC dc(this);
	dc.SetBackground(wxBrush(GetBackgroundColour()));
	dc.Clear();
	if (bitmap.IsOk())
		dc.DrawBitmap(bitmap.GetBitmap(GetScaledBitmapSize()), wxPoint(0, 0), true);
}

wxSize BigStaticBitmapCtrl::DoGetBestSize() const
{
	if (bitmap.IsOk())
		return GetScaledBitmapSize();

	return wxPanel::DoGetBestSize();
}

wxSize BigStaticBitmapCtrl::GetScaledBitmapSize() const
{
	return FromDIP(bitmap_logical_size);
}

void BigStaticBitmapCtrl::UpdateBitmapSize()
{
	const wxSize bitmap_size = GetScaledBitmapSize();
	InvalidateBestSize();
	SetInitialSize(bitmap_size);
	SetMinSize(bitmap_size);
}

void BigStaticBitmapCtrl::OnDPIChanged(wxDPIChangedEvent& event)
{
	UpdateBitmapSize();
	if (wxWindow *parent = GetParent())
		parent->Layout();
	Refresh();
	event.Skip();
}

void BigStaticBitmapCtrl::OnMouseLeftDown(wxMouseEvent &event)
{
	if (window_to_drag != NULL)
	{
		dragpoint = event.GetPosition();
	}
	CaptureMouse();
}

void BigStaticBitmapCtrl::OnMouseLeftUp(wxMouseEvent &event)
{
	ReleaseMouse();
}

void BigStaticBitmapCtrl::OnMouseMove(wxMouseEvent &event)
{
	if (window_to_drag != NULL && event.Dragging() && HasCapture())
	{
		wxPoint npoint(event.GetPosition());
		wxPoint wndpos = window_to_drag->GetScreenPosition();
		wxPoint thispos = this->GetScreenPosition();
		//ReleaseMouse();
		window_to_drag->Move(wndpos.x + npoint.x - dragpoint.x,
							 wndpos.y + npoint.y - dragpoint.y);
		//CaptureMouse();
		if (thispos == this->GetScreenPosition()) // if this ctrl did not move when window_to_drag moved
			dragpoint = npoint;
	}
	event.Skip(true);
}
