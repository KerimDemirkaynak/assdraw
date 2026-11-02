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
// Name:        settings.cpp
// Purpose:     settings property grid
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////

#include "assdraw.hpp"
#include "settings.hpp"

wxDEFINE_EVENT(wxEVT_SETTINGS_CHANGED, wxCommandEvent);

// ----------------------------------------------------------------------------
// ASSDrawSettingsDialog
// ----------------------------------------------------------------------------

ASSDrawSettingsDialog::ASSDrawSettingsDialog(wxWindow *parent, ASSDrawFrame *frame, int id)
 : wxPanel(parent, id)
{
	m_frame = frame;
	propgrid = NULL;
}

void ASSDrawSettingsDialog::Init()
{
    propgrid = new wxPropertyGrid(this,
		wxID_ANY,
		wxDefaultPosition,
		wxDefaultSize,
		//wxPG_BOLD_MODIFIED |
		//wxPG_SPLITTER_AUTO_CENTER |
		//wxPG_AUTO_SORT |
		//wxPG_HIDE_MARGIN | wxPG_STATIC_SPLITTER |
		wxPG_TOOLTIPS |
		//wxPG_NOCATEGORIES |
		wxTAB_TRAVERSAL //|
		//wxSUNKEN_BORDER
	);

    #define APPENDCOLOURPROP(pgid, label, color) pgid = propgrid->Append(new wxColourProperty(label, wxPG_LABEL, color));
    #define APPENDUINTPROP(pgid, label, uint) \
		pgid = propgrid->Append(new wxUIntProperty(label, wxPG_LABEL, uint) ); \
		propgrid->SetPropertyValidator( pgid, validator );
    #define APPENDBOOLPROP(pgid, label, boolvar) \
	    pgid = propgrid->Append(new wxBoolProperty (label, wxPG_LABEL, boolvar ) ); \
    	propgrid->SetPropertyAttribute( pgid, wxPG_BOOL_USE_CHECKBOX, (long)1 );
	wxLongPropertyValidator validator(0x0,0xFF);

    appearance_category = propgrid->Append(new wxPropertyCategory(_("Appearance"), "appearance"));
	APPENDCOLOURPROP(colors_canvas_bg_pgid, _("Canvas"), m_frame->colors.canvas_bg)
	APPENDCOLOURPROP(colors_canvas_shape_normal_pgid, _("Drawing"), m_frame->colors.canvas_shape_normal)
	APPENDUINTPROP(alphas_canvas_shape_normal_pgid, _("Drawing @"), m_frame->alphas.canvas_shape_normal)
	APPENDCOLOURPROP(colors_canvas_shape_preview_pgid, _("Preview"), m_frame->colors.canvas_shape_preview)
	APPENDUINTPROP(alphas_canvas_shape_preview_pgid, _("Preview @"), m_frame->alphas.canvas_shape_preview)
	APPENDCOLOURPROP(colors_canvas_shape_outline_pgid, _("Outline"), m_frame->colors.canvas_shape_outline)
	APPENDUINTPROP(alphas_canvas_shape_outline_pgid, _("Outline @"), m_frame->alphas.canvas_shape_outline)
	APPENDCOLOURPROP(colors_canvas_shape_guideline_pgid, _("Control lines"), m_frame->colors.canvas_shape_guideline)
	APPENDUINTPROP(alphas_canvas_shape_guideline_pgid, _("Control lines @"), m_frame->alphas.canvas_shape_guideline)
	APPENDCOLOURPROP(colors_canvas_shape_mainpoint_pgid, _("Points"), m_frame->colors.canvas_shape_mainpoint)
	APPENDUINTPROP(alphas_canvas_shape_mainpoint_pgid, _("Points @"), m_frame->alphas.canvas_shape_mainpoint)
	APPENDCOLOURPROP(colors_canvas_shape_controlpoint_pgid, _("Control points"), m_frame->colors.canvas_shape_controlpoint)
	APPENDUINTPROP(alphas_canvas_shape_controlpoint_pgid, _("Control points @"), m_frame->alphas.canvas_shape_controlpoint)
	APPENDCOLOURPROP(colors_canvas_shape_selectpoint_pgid, _("Selected points"), m_frame->colors.canvas_shape_selectpoint)
	APPENDUINTPROP(alphas_canvas_shape_selectpoint_pgid, _("Selected points @"), m_frame->alphas.canvas_shape_selectpoint)
	APPENDCOLOURPROP(colors_library_libarea_pgid, _("Library"), m_frame->colors.library_libarea)
	APPENDCOLOURPROP(colors_library_shape_pgid, _("Library shapes"), m_frame->colors.library_shape)
	APPENDCOLOURPROP(colors_origin_pgid, _("Origin"), m_frame->colors.origin)
	APPENDUINTPROP(sizes_origincross_pgid, _("Origin cross size"), m_frame->sizes.origincross)
	APPENDCOLOURPROP(colors_ruler_h_pgid, _("H ruler"), m_frame->colors.ruler_h)
	APPENDCOLOURPROP(colors_ruler_v_pgid, _("V ruler"), m_frame->colors.ruler_v)

	behavior_category = propgrid->Append(new wxPropertyCategory(_("Behaviors"), "behaviors"));
	APPENDBOOLPROP(behaviors_capitalizecmds_pgid, _("Capitalize commands"), m_frame->behaviors.capitalizecmds);
	APPENDBOOLPROP(behaviors_autoaskimgopac_pgid, _("Ask for image opacity"), m_frame->behaviors.autoaskimgopac);
	APPENDUINTPROP(alphas_dfltimgopac_pgid, _("Default image opacity (0-255)"), m_frame->alphas.dfltimgopac)
	APPENDBOOLPROP(behaviors_parse_spc_pgid, _("Parse S/P/C"), m_frame->behaviors.parse_spc);
	APPENDBOOLPROP(behaviors_nosplashscreen_pgid, _("No splash screen"), m_frame->behaviors.nosplashscreen);
	APPENDBOOLPROP(behaviors_confirmquit_pgid, _("Confirm quit"), m_frame->behaviors.confirmquit);

	wxFlexGridSizer *sizer = new wxFlexGridSizer(2, 1, 0, 0);
	sizer->AddGrowableCol(0);
	sizer->AddGrowableRow(0);
	sizer->Add(propgrid, 1, wxEXPAND);

	wxBoxSizer *bsizer = new wxBoxSizer(wxHORIZONTAL);
	wxButton *abutton = apply_button = new wxButton(this, wxID_APPLY, _("Apply"));
	abutton->Connect(wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler(ASSDrawSettingsDialog::OnSettingsApplyButtonClicked), NULL, this);
	bsizer->Add(abutton, 2, wxEXPAND);
	wxButton *rbutton = revert_button = new wxButton(this, wxID_REVERT, _("Revert"));
	rbutton->Connect(wxEVT_COMMAND_BUTTON_CLICKED, wxCommandEventHandler(ASSDrawSettingsDialog::OnSettingsRevertButtonClicked), NULL, this);
	bsizer->Add(rbutton, 1, wxEXPAND);
	bsizer->Layout();

	sizer->Add(bsizer, 1, wxEXPAND);
	sizer->Layout();
	SetSizer(sizer);
}

ASSDrawSettingsDialog::~ASSDrawSettingsDialog()
{
	if (propgrid) propgrid->Clear();
}

void ASSDrawSettingsDialog::OnSettingsApplyButtonClicked(wxCommandEvent &event)
{

	wxObject *event_object = event.GetEventObject();
	if (propgrid == NULL) return;

	#define PARSECOLOR(color, pgid) \
	{ \
		wxVariant variant = propgrid->GetPropertyValue(pgid); \
		wxColour value; \
		value << variant; \
		if (value.IsOk()) color = value; \
	}

	#define PARSE(ptr, pgid) propgrid->GetPropertyValue(pgid).Convert(ptr);

	PARSECOLOR(m_frame->colors.canvas_bg, colors_canvas_bg_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_controlpoint, colors_canvas_shape_controlpoint_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_guideline, colors_canvas_shape_guideline_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_mainpoint, colors_canvas_shape_mainpoint_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_normal, colors_canvas_shape_normal_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_outline, colors_canvas_shape_outline_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_preview, colors_canvas_shape_preview_pgid)
	PARSECOLOR(m_frame->colors.canvas_shape_selectpoint, colors_canvas_shape_selectpoint_pgid)
	PARSECOLOR(m_frame->colors.library_libarea, colors_library_libarea_pgid)
	PARSECOLOR(m_frame->colors.library_shape, colors_library_shape_pgid)
	PARSECOLOR(m_frame->colors.origin, colors_origin_pgid)
	PARSECOLOR(m_frame->colors.ruler_h, colors_ruler_h_pgid)
	PARSECOLOR(m_frame->colors.ruler_v, colors_ruler_v_pgid)

	PARSE(&m_frame->alphas.canvas_shape_controlpoint, alphas_canvas_shape_controlpoint_pgid)
	PARSE(&m_frame->alphas.canvas_shape_guideline, alphas_canvas_shape_guideline_pgid)
	PARSE(&m_frame->alphas.canvas_shape_mainpoint, alphas_canvas_shape_mainpoint_pgid)
	PARSE(&m_frame->alphas.canvas_shape_normal, alphas_canvas_shape_normal_pgid)
	PARSE(&m_frame->alphas.canvas_shape_outline, alphas_canvas_shape_outline_pgid)
	PARSE(&m_frame->alphas.canvas_shape_preview, alphas_canvas_shape_preview_pgid)
	PARSE(&m_frame->alphas.canvas_shape_selectpoint, alphas_canvas_shape_selectpoint_pgid)
	PARSE(&m_frame->alphas.dfltimgopac, alphas_dfltimgopac_pgid)

	PARSE(&m_frame->sizes.origincross, sizes_origincross_pgid)

	PARSE(&m_frame->behaviors.autoaskimgopac, behaviors_autoaskimgopac_pgid)
	PARSE(&m_frame->behaviors.capitalizecmds, behaviors_capitalizecmds_pgid)
	PARSE(&m_frame->behaviors.parse_spc, behaviors_parse_spc_pgid)
	PARSE(&m_frame->behaviors.nosplashscreen, behaviors_nosplashscreen_pgid)
	PARSE(&m_frame->behaviors.confirmquit, behaviors_confirmquit_pgid)

	wxCommandEvent evento( wxEVT_SETTINGS_CHANGED, event.GetId() );
	evento.SetEventObject(event_object);
    m_frame->GetEventHandler()->ProcessEvent( evento );
	propgrid->ClearModifiedStatus();

}

void ASSDrawSettingsDialog::OnSettingsRevertButtonClicked(wxCommandEvent &event)
{
	RefreshSettingsDisplay();
	propgrid->ClearModifiedStatus();
}

void ASSDrawSettingsDialog::RefreshThemeViewColors()
{
	std::pair<wxPGId, wxColour> fields[] = {
		{colors_canvas_bg_pgid, m_frame->colors.canvas_bg},
		{colors_canvas_shape_controlpoint_pgid, m_frame->colors.canvas_shape_controlpoint},
		{colors_canvas_shape_guideline_pgid, m_frame->colors.canvas_shape_guideline},
		{colors_canvas_shape_mainpoint_pgid, m_frame->colors.canvas_shape_mainpoint},
		{colors_canvas_shape_selectpoint_pgid, m_frame->colors.canvas_shape_selectpoint},
		{colors_library_libarea_pgid, m_frame->colors.library_libarea},
		{colors_library_shape_pgid, m_frame->colors.library_shape},
		{colors_origin_pgid, m_frame->colors.origin},
		{colors_ruler_h_pgid, m_frame->colors.ruler_h},
		{colors_ruler_v_pgid, m_frame->colors.ruler_v}
	};
	for (auto& field : fields)
		if (!field.first->HasFlag(wxPGFlags::Modified))
			propgrid->SetPropertyValue(field.first, field.second);
}

void ASSDrawSettingsDialog::RefreshSettingsDisplay()
{
	if (propgrid == NULL) return;

	#define UPDATESETTING(value, pgid) propgrid->SetPropertyValue(pgid, value);

	UPDATESETTING(m_frame->colors.canvas_bg, colors_canvas_bg_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_controlpoint, colors_canvas_shape_controlpoint_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_guideline, colors_canvas_shape_guideline_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_mainpoint, colors_canvas_shape_mainpoint_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_normal, colors_canvas_shape_normal_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_outline, colors_canvas_shape_outline_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_preview, colors_canvas_shape_preview_pgid)
	UPDATESETTING(m_frame->colors.canvas_shape_selectpoint, colors_canvas_shape_selectpoint_pgid)
	UPDATESETTING(m_frame->colors.library_libarea, colors_library_libarea_pgid)
	UPDATESETTING(m_frame->colors.library_shape, colors_library_shape_pgid)
	UPDATESETTING(m_frame->colors.origin, colors_origin_pgid)
	UPDATESETTING(m_frame->colors.ruler_h, colors_ruler_h_pgid)
	UPDATESETTING(m_frame->colors.ruler_v, colors_ruler_v_pgid)

	UPDATESETTING(m_frame->alphas.canvas_shape_controlpoint, alphas_canvas_shape_controlpoint_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_guideline, alphas_canvas_shape_guideline_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_mainpoint, alphas_canvas_shape_mainpoint_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_normal, alphas_canvas_shape_normal_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_outline, alphas_canvas_shape_outline_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_preview, alphas_canvas_shape_preview_pgid)
	UPDATESETTING(m_frame->alphas.canvas_shape_selectpoint, alphas_canvas_shape_selectpoint_pgid)
	UPDATESETTING(m_frame->alphas.dfltimgopac, alphas_dfltimgopac_pgid)

	UPDATESETTING(m_frame->sizes.origincross, sizes_origincross_pgid)

	UPDATESETTING(m_frame->behaviors.capitalizecmds, behaviors_capitalizecmds_pgid)
	UPDATESETTING(m_frame->behaviors.autoaskimgopac, behaviors_autoaskimgopac_pgid)
	UPDATESETTING(m_frame->behaviors.parse_spc, behaviors_parse_spc_pgid)
	UPDATESETTING(m_frame->behaviors.nosplashscreen, behaviors_nosplashscreen_pgid)
	UPDATESETTING(m_frame->behaviors.confirmquit, behaviors_confirmquit_pgid)
}

void ASSDrawSettingsDialog::UpdateTranslations()
{
	appearance_category->SetLabel(_("Appearance"));
	behavior_category->SetLabel(_("Behaviors"));
	colors_canvas_bg_pgid->SetLabel(_("Canvas"));
	colors_canvas_shape_normal_pgid->SetLabel(_("Drawing"));
	alphas_canvas_shape_normal_pgid->SetLabel(_("Drawing @"));
	colors_canvas_shape_preview_pgid->SetLabel(_("Preview"));
	alphas_canvas_shape_preview_pgid->SetLabel(_("Preview @"));
	colors_canvas_shape_outline_pgid->SetLabel(_("Outline"));
	alphas_canvas_shape_outline_pgid->SetLabel(_("Outline @"));
	colors_canvas_shape_guideline_pgid->SetLabel(_("Control lines"));
	alphas_canvas_shape_guideline_pgid->SetLabel(_("Control lines @"));
	colors_canvas_shape_mainpoint_pgid->SetLabel(_("Points"));
	alphas_canvas_shape_mainpoint_pgid->SetLabel(_("Points @"));
	colors_canvas_shape_controlpoint_pgid->SetLabel(_("Control points"));
	alphas_canvas_shape_controlpoint_pgid->SetLabel(_("Control points @"));
	colors_canvas_shape_selectpoint_pgid->SetLabel(_("Selected points"));
	alphas_canvas_shape_selectpoint_pgid->SetLabel(_("Selected points @"));
	colors_library_libarea_pgid->SetLabel(_("Library"));
	colors_library_shape_pgid->SetLabel(_("Library shapes"));
	colors_origin_pgid->SetLabel(_("Origin"));
	sizes_origincross_pgid->SetLabel(_("Origin cross size"));
	colors_ruler_h_pgid->SetLabel(_("H ruler"));
	colors_ruler_v_pgid->SetLabel(_("V ruler"));
	behaviors_capitalizecmds_pgid->SetLabel(_("Capitalize commands"));
	behaviors_autoaskimgopac_pgid->SetLabel(_("Ask for image opacity"));
	alphas_dfltimgopac_pgid->SetLabel(_("Default image opacity (0-255)"));
	behaviors_parse_spc_pgid->SetLabel(_("Parse S/P/C"));
	behaviors_nosplashscreen_pgid->SetLabel(_("No splash screen"));
	behaviors_confirmquit_pgid->SetLabel(_("Confirm quit"));
	apply_button->SetLabel(_("Apply"));
	revert_button->SetLabel(_("Revert"));
	propgrid->Refresh();
	Layout();
}
