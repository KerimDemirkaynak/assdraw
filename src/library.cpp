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

#include "assdraw.hpp"
#include "library.hpp"

#include <wx/clipbrd.h>

#if !defined(__WINDOWS__)
#include "xpm/res.h"
#endif

BEGIN_EVENT_TABLE(ASSDrawShapePreview, ASSDrawEngine)
    EVT_SIZE(ASSDrawShapePreview::OnSize)
END_EVENT_TABLE()

ASSDrawShapePreview::ASSDrawShapePreview( wxWindow *parent, ASSDrawShapeLibrary *_shapelib, wxString initialcmds )
	: ASSDrawEngine(parent, wxSIMPLE_BORDER)
{
	shapelib = _shapelib;
	if (ParseASS(initialcmds) > 0)
		SetFitToViewPointOnNextPaint(FromDIP(5), FromDIP(5));
	cb = new wxCheckBox(this, wxID_ANY, _T(""));
}

void ASSDrawShapePreview::OnSize(wxSizeEvent& event)
{
	return;
	wxSize siz = event.GetSize();

	if (shapelib->layout == HORIZONTAL)
		SetSize(siz.x, siz.x);
	else
		SetSize(siz.y, siz.y);
	SetFitToViewPointOnNextPaint(FromDIP(10), FromDIP(10));
}

BEGIN_EVENT_TABLE(ASSDrawShapeLibrary, wxScrolledWindow)
    EVT_DPI_CHANGED(ASSDrawShapeLibrary::OnDPIChanged)
    EVT_SIZE(ASSDrawShapeLibrary::OnSize)
    EVT_MENU_RANGE(MENU_RANGE_START, MENU_RANGE_END, ASSDrawShapeLibrary::OnPopupMenuClicked)
    EVT_TOOL(TOOL_SAVE, ASSDrawShapeLibrary::SaveShapeFromCanvas)
    EVT_TOOL_RANGE(TOOL_CHECK, TOOL_UNCHECK, ASSDrawShapeLibrary::CheckUncheckAllPreviews)
    EVT_TOOL(TOOL_DELETE, ASSDrawShapeLibrary::DeleteChecked)
END_EVENT_TABLE()

ASSDrawShapeLibrary::ASSDrawShapeLibrary( wxWindow *parent, ASSDrawFrame *frame )
	: wxScrolledWindow(parent, wxID_ANY)
{
	m_frame = frame;
	//sizer = NULL;
	layout = VERTICAL;

	toolbar = new wxToolBar(this, wxID_ANY, __DPDS__ , wxTB_HORIZONTAL | wxNO_BORDER | wxTB_FLAT | wxTB_NODIVIDER);
	toolbar->SetToolBitmapSize(FromDIP(wxSize(16, 16)));
	toolbar->SetMargins(0, FromDIP(3));
	toolbar->AddTool(TOOL_SAVE, _T("Save canvas"), ASSDRAW_BITMAP_BUNDLE(add));
	toolbar->AddSeparator();
	toolbar->AddTool(TOOL_CHECK, _T("Select all"), ASSDRAW_BITMAP_BUNDLE(check));
	toolbar->AddTool(TOOL_UNCHECK, _T("Select none"), ASSDRAW_BITMAP_BUNDLE(uncheck));
	toolbar->AddTool(TOOL_DELETE, _T("Delete selected"), ASSDRAW_BITMAP_BUNDLE(delcross));

	libarea = new wxScrolledWindow(this, wxID_ANY, __DPDS__ , wxScrolledWindowStyle | wxSIMPLE_BORDER);
	libarea->SetBackgroundColour(wxColour(0xFF, 0xFF, 0x99));
	sizer = new wxFlexGridSizer(0, 1, 0, 0);
	((wxFlexGridSizer*) sizer)->AddGrowableCol(0);
	libarea->SetSizer(sizer);
	libarea->SetScrollbars(FromDIP(20), FromDIP(20), 50, 50);
	libsizer = new wxFlexGridSizer(2, 1, 0, 0);
	libsizer->AddGrowableCol(0);
	libsizer->AddGrowableRow(1);
	libsizer->Add(toolbar, 0, wxBOTTOM, FromDIP(2));
	libsizer->Add(libarea,1,wxEXPAND);
	toolbar->Realize();
	libsizer->Layout();

	SetSizer(libsizer);
}

void ASSDrawShapeLibrary::OnDPIChanged(wxDPIChangedEvent& event)
{
	UpdateDpiLayout();
	event.Skip();
}

void ASSDrawShapeLibrary::OnSize(wxSizeEvent& event)
{
	if (sizer == NULL || sizer->GetChildren().size() == 0) return;

	wxSize siz = GetClientSize();
	libsizer->SetDimension(0, 0, siz.x, siz.y);

	UpdatePreviewDisplays();
}

void ASSDrawShapeLibrary::UpdateDpiLayout()
{
	if (toolbar)
	{
		toolbar->SetToolBitmapSize(FromDIP(wxSize(16, 16)));
		toolbar->SetMargins(0, FromDIP(3));
		toolbar->Realize();
	}

	if (libsizer && libsizer->GetItem((size_t)0))
		libsizer->GetItem((size_t)0)->SetBorder(FromDIP(2));

	if (libarea)
		libarea->SetScrollbars(FromDIP(20), FromDIP(20), 50, 50);

	if (sizer)
	{
		wxwxSizerItemListNode *node = sizer->GetChildren().GetFirst();
		while (node != NULL)
		{
			node->GetData()->SetBorder(FromDIP(5));
			node = node->GetNext();
		}
	}

	UpdatePreviewDisplays();
	Layout();
}

ASSDrawShapePreview* ASSDrawShapeLibrary::AddShapePreview(wxString cmds, bool addtotop)
{
	ASSDrawShapePreview *prev = new ASSDrawShapePreview(libarea, this, cmds);
	prev->Connect(wxEVT_LEFT_DCLICK, wxMouseEventHandler(ASSDrawShapeLibrary::OnMouseLeftDClick), NULL, this);
	prev->Connect(wxEVT_RIGHT_UP, wxMouseEventHandler(ASSDrawShapeLibrary::OnMouseRightClick), NULL, this);
	ASSDrawFrame::wxColourToAggRGBA(m_frame->colors.library_shape, prev->rgba_shape);
	if (addtotop)
		sizer->Insert(0, prev, 1, wxEXPAND | wxTOP | wxLEFT | wxRIGHT, FromDIP(5));
	else
		sizer->Add(prev, 1, wxEXPAND | wxTOP | wxLEFT | wxRIGHT, FromDIP(5));
	UpdatePreviewDisplays();
	return prev;
}

void ASSDrawShapeLibrary::UpdatePreviewDisplays()
{
	wxSize siz = libarea->GetClientSize();
	int dim = siz.x - FromDIP(15);
	if (dim < 1)
		dim = 1;
	libarea->Show(false);
	wxwxSizerItemListNode *node = sizer->GetChildren().GetFirst();
	while (node != NULL)
	{
		ASSDrawShapePreview *shprvw = (ASSDrawShapePreview *) node->GetData()->GetWindow();
		shprvw->SetSize(dim, dim);
		sizer->SetItemMinSize(shprvw, dim - FromDIP(20), dim - FromDIP(20));
		shprvw->SetFitToViewPointOnNextPaint(FromDIP(10), FromDIP(10));
		shprvw->Refresh();
		node = node->GetNext();
	}
	sizer->Layout();
	sizer->FitInside(libarea);
	libarea->Show(true);
}

void ASSDrawShapeLibrary::OnMouseLeftDClick(wxMouseEvent &event)
{
	ASSDrawShapePreview *preview = (ASSDrawShapePreview *) event.GetEventObject();
	LoadToCanvas(preview);
}

void ASSDrawShapeLibrary::OnMouseRightClick(wxMouseEvent &event)
{
	ASSDrawShapePreview *prev = (ASSDrawShapePreview *) event.GetEventObject();
	if (prev && prev->shapelib == this)
	{
		activepreview = prev;
		wxMenu *menu = new wxMenu;
		wxMenuItem *menuload = new wxMenuItem(menu, MENU_LOAD, _("Load to canvas"));
#ifdef __WINDOWS__
		wxFont f = menuload->GetFont();
		f.SetWeight(wxFONTWEIGHT_BOLD);
		menuload->SetFont(f);
#endif
		menu->Append(menuload);
		//menu->Append(MENU_LOAD, _("Load to canvas"))->GetFont().SetWeight(wxFONTWEIGHT_BOLD);
		menu->Append(MENU_COPYCLIPBOARD, _("Copy commands to clipboard"));
		menu->Append(MENU_SAVECANVAS, _("Save canvas here"));
		wxMenu *submenu = new wxMenu;
		submenu->Append(MENU_DELETE, _("Confirm delete?"));
		menu->Append(MENU_DUMMY, _("Delete from library"), submenu);
		PopupMenu(menu);
		delete menu;
	}
}

void ASSDrawShapeLibrary::OnPopupMenuClicked(wxCommandEvent &event)
{
	int id = event.GetId();
	switch(id)
	{
	case MENU_LOAD:
		LoadToCanvas(activepreview);
		break;
	case MENU_COPYCLIPBOARD:
		if (wxTheClipboard->Open())
		{
			if (wxTheClipboard->IsSupported( wxDF_TEXT ))
			{
				wxTheClipboard->SetData( new wxTextDataObject( activepreview->GenerateASS() ) );
			}
			wxTheClipboard->Close();
		}
		break;
	case MENU_SAVECANVAS:
		activepreview->ParseASS(m_frame->m_canvas->GenerateASS());
		activepreview->SetFitToViewPointOnNextPaint();
		activepreview->RefreshDisplay();
		break;
	case MENU_DELETE:
		sizer->Detach(activepreview);
		activepreview->Show(false);
		activepreview->Destroy();
		UpdatePreviewDisplays();
		Refresh();
		break;
	}
}

void ASSDrawShapeLibrary::SaveShapeFromCanvas(wxCommandEvent& WXUNUSED(event))
{
	AddShapePreview(m_frame->m_canvas->GenerateASS(), true);
}

void ASSDrawShapeLibrary::CheckUncheckAllPreviews(wxCommandEvent &event)
{
	bool checked = event.GetId() == TOOL_CHECK;
	wxwxSizerItemListNode *node = sizer->GetChildren().GetFirst();
	while (node != NULL)
	{
		ASSDrawShapePreview *shprvw = (ASSDrawShapePreview *) node->GetData()->GetWindow();
		shprvw->cb->SetValue(checked);
		node = node->GetNext();
	}
}

void ASSDrawShapeLibrary::DeleteChecked(wxCommandEvent& WXUNUSED(event))
{
	wxwxSizerItemListNode *node = sizer->GetChildren().GetFirst();
	while (node != NULL)
	{
		ASSDrawShapePreview *shprvw = (ASSDrawShapePreview *) node->GetData()->GetWindow();
		if (shprvw->cb->GetValue())
		{
			sizer->Detach(shprvw);
			shprvw->Show(false);
			shprvw->Destroy();
		}
		node = node->GetNext();
	}
	UpdatePreviewDisplays();
	Refresh();
}

std::vector< ASSDrawShapePreview *> ASSDrawShapeLibrary::GetShapePreviews()
{
	std::vector< ASSDrawShapePreview *> out;
	wxwxSizerItemListNode *node = sizer->GetChildren().GetFirst();
	while (node != NULL)
	{
		ASSDrawShapePreview *shprvw = (ASSDrawShapePreview *) node->GetData()->GetWindow();
		out.push_back(shprvw);
		node = node->GetNext();
	}
	return out;
}

void ASSDrawShapeLibrary::LoadToCanvas(ASSDrawShapePreview *preview)
{
	m_frame->m_canvas->AddUndo(_("Load shape from library"));
	m_frame->m_canvas->ParseASS(preview->GenerateASS());
	m_frame->m_canvas->RefreshDisplay();
	m_frame->m_canvas->RefreshUndocmds();
	m_frame->UpdateFrameUI();
}
