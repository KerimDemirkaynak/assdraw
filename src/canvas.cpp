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
// Name:        canvas.cpp
// Purpose:     implementations of ASSDraw main canvas class
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////

#include "assdraw.hpp"
#include "cmd.hpp"
#include "styled_ass.hpp"
#include "agg_gsv_text.h"
#include "agg_ellipse.h"
#include "agg_conv_dash.h"
#include "agg_trans_bilinear.h"
#include "agg_trans_perspective.h"

#include "agghelper.hpp"
#include <algorithm>
#include <math.h>
#include <cmath>
#include <wx/colordlg.h>
#include <wx/image.h>
#include <wx/filename.h>

namespace {
constexpr double kEditorPointMinimumDiameter = 8.0;
constexpr double kEditorPointMaximumDiameter = 14.0;
constexpr double kEditorPointHitRadius = 9.0;
constexpr double kTransformHandleRadius = 8.0;

AssDrawingCommand ToAssDrawingCommand(const DrawCmd* command)
{
	AssDrawingCommand serialized;
	serialized.type = "mnlbspc"[static_cast<int>(command->type)];

	if (command->type == B)
	{
		if (command->initialized && command->controlpoints.size() >= 2)
		{
			PointList::const_iterator control = command->controlpoints.begin();
			serialized.points.push_back({ (*control)->x(), (*control)->y() });
			++control;
			serialized.points.push_back({ (*control)->x(), (*control)->y() });
		}
		else
			serialized.unknown_point_pairs = 2;
	}
	else if (command->type == S)
	{
		if (command->initialized)
		{
			for (PointList::const_iterator control = command->controlpoints.begin();
				control != command->controlpoints.end(); ++control)
				serialized.points.push_back({ (*control)->x(), (*control)->y() });
		}
		else
			serialized.unknown_point_pairs = command->controlpoints.size();
		serialized.close_spline = static_cast<const DrawCmd_S*>(command)->closed;
	}

	if (command->m_point)
		serialized.points.push_back({ command->m_point->x(), command->m_point->y() });
	return serialized;
}

double EditorPointDiameter(double scale)
{
	if (scale < kEditorPointMinimumDiameter)
		return kEditorPointMinimumDiameter;
	if (scale > kEditorPointMaximumDiameter)
		return kEditorPointMaximumDiameter;
	return scale;
}

double SignedArea(const std::vector<wxRealPoint>& contour)
{
	double area = 0.0;
	for (std::size_t index = 0; index < contour.size(); ++index)
	{
		const wxRealPoint& from = contour[index];
		const wxRealPoint& to = contour[(index + 1) % contour.size()];
		area += from.x * to.y - to.x * from.y;
	}
	return area * 0.5;
}

int WindingAtPoint(const std::vector<wxRealPoint>& contour, const wxRealPoint& point)
{
	int winding = 0;
	if (contour.size() < 3)
		return winding;

	for (std::size_t index = 0; index < contour.size(); ++index)
	{
		const wxRealPoint& from = contour[index];
		const wxRealPoint& to = contour[(index + 1) % contour.size()];
		const double side = (to.x - from.x) * (point.y - from.y) -
			(to.y - from.y) * (point.x - from.x);
		if (from.y <= point.y)
		{
			if (to.y > point.y && side > 0.0)
				++winding;
		}
		else if (to.y <= point.y && side < 0.0)
		{
			--winding;
		}
	}
	return winding;
}

bool IsInsideContour(const std::vector<wxRealPoint>& contour, const wxRealPoint& point)
{
	return WindingAtPoint(contour, point) != 0;
}

agg::rgba ToAggColor(AssRgb color, std::uint8_t opacity)
{
	return agg::rgba(color.red / 255.0, color.green / 255.0,
		color.blue / 255.0, opacity / 255.0);
}
}

// ----------------------------------------------------------------------------
// the main drawing canvas: ASSDrawCanvas
// ----------------------------------------------------------------------------

BEGIN_EVENT_TABLE(ASSDrawCanvas, ASSDrawEngine)
    EVT_MOTION (ASSDrawCanvas::OnMouseMove)
    EVT_LEFT_UP(ASSDrawCanvas::OnMouseLeftUp)
    EVT_LEFT_DOWN(ASSDrawCanvas::OnMouseLeftDown)
	EVT_LEFT_DCLICK(ASSDrawCanvas::OnMouseLeftDClick)
    EVT_RIGHT_UP(ASSDrawCanvas::OnMouseRightUp)
    EVT_RIGHT_DOWN(ASSDrawCanvas::OnMouseRightDown)
    EVT_RIGHT_DCLICK(ASSDrawCanvas::OnMouseRightDClick)
	EVT_MOUSEWHEEL(ASSDrawCanvas::OnMouseWheel)
	EVT_DPI_CHANGED(ASSDrawCanvas::OnDPIChanged)
    EVT_KEY_DOWN(ASSDrawCanvas::CustomOnKeyDown)
    EVT_KEY_UP(ASSDrawCanvas::CustomOnKeyUp)
	EVT_MENU(MENU_DRC_LNTOBEZ, ASSDrawCanvas::OnSelect_ConvertLineToBezier)
	EVT_MENU(MENU_DRC_BEZTOLN, ASSDrawCanvas::OnSelect_ConvertBezierToLine)
	EVT_MENU(MENU_DRC_C1CONTBEZ, ASSDrawCanvas::OnSelect_C1ContinuityBezier)
	EVT_MENU(MENU_DRC_MOVE00, ASSDrawCanvas::OnSelect_Move00Here)
	EVT_MOUSE_CAPTURE_LOST(ASSDrawCanvas::CustomOnMouseCaptureLost)
END_EVENT_TABLE()

ASSDrawCanvas::ASSDrawCanvas(wxWindow *parent, ASSDrawFrame *frame, int extraflags)
 : ASSDrawEngine( parent, extraflags )
{
	m_frame = frame;
	preview_mode = false;
    lastDrag_left = NULL;
    lastDrag_right = NULL;
    dragAnchor_left = NULL;
    dragAnchor_right = NULL;
    newcommand = NULL;
    mousedownAt_point = NULL;
    pointedAt_point = NULL;
    draw_mode = MODE_ARR;

    dragOrigin = false;
	hilite_cmd = NULL;
	hilite_point = NULL;
	capturemouse_left = false;
	capturemouse_right = false;
	bgimg.bgimg = NULL;
	bgimg.alpha = 0.5;
	rectbound2upd = -1, rectbound2upd2 = -1;
	coloring_target_selected = false;
	coloring_target_shape = NULL;

	rgba_shape_normal = agg::rgba(0,0,1,0.5);
	rgba_outline = agg::rgba(0,0,0);
	rgba_guideline = agg::rgba(0.5,0.5,0.5);
	rgba_mainpoint = agg::rgba(1,0,0,0.75);
	rgba_controlpoint = agg::rgba(0,1,0,0.75);
	rgba_selectpoint = agg::rgba(0,0,1,0.75);
	rgba_origin = agg::rgba(0,0,0);
	rgba_ruler_h = agg::rgba(0,0,1);
	rgba_ruler_v = agg::rgba(1,0,0);
	ApplyShapeStyle();

	wxFlexGridSizer* sizer = new wxFlexGridSizer(1, 1, 0);
    sizer->AddGrowableRow(0);
    sizer->AddGrowableCol(0);
    sizer->Add( this, 0, wxGROW|wxGROW, 5);
    parent->SetSizer(sizer);

	// for background image loading
	::wxInitAllImageHandlers();
	bgimg.bgimg = NULL;
	// drag image background file
	SetDropTarget(new ASSDrawFileDropTarget(this));

	hasStatusBar = m_frame->GetStatusBar() != NULL;

	// cursor = crosshair
	SetCursor( *wxCROSS_CURSOR );

	bgimg.alpha_dlg = new wxDialog(this, wxID_ANY, wxString(_("Background image opacity")));
	bgimg.alpha_slider = new wxSlider(bgimg.alpha_dlg, TB_BGALPHA_SLIDER, 50, 0, 100, __DPDS__ , wxSL_LABELS);
	bgimg.alpha_slider->SetMinSize(wxSize(FromDIP(280), -1));
	bgimg.alpha_dlg->Fit();
	bgimg.alpha_dlg->Show(false);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_LINEUP, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_LINEDOWN, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_PAGEUP, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_PAGEDOWN, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_THUMBTRACK, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);
	bgimg.alpha_slider->Connect(wxEVT_SCROLL_CHANGED, wxScrollEventHandler(ASSDrawCanvas::OnAlphaSliderChanged), NULL, this);

	RefreshUndocmds();

}

// Destructor
ASSDrawCanvas::~ASSDrawCanvas()
{
	ASSDrawEngine::ResetEngine(false);
	if (bgimg.bgimg) delete bgimg.bgimg;
}

void ASSDrawCanvas::ParseASS(wxString str, bool addundo)
{
	if (addundo)
		AddUndo(wxTRANSLATE("Modify drawing commands"));

	const std::string source(str.mb_str(wxConvUTF8));
	const StyledAssImport imported = ImportStyledAss(source, shape_style);

	subshape_styles.clear();
	coloring_target_shape = NULL;
	coloring_target_selected = false;
	ASSDrawEngine::ParseASS(wxString(imported.drawing_text.c_str(), wxConvUTF8));
	std::size_t style_index = 0;
	for (DrawCmdList::iterator command = cmds.begin(); command != cmds.end(); ++command)
	{
		if ((*command)->type != M)
			continue;
		subshape_styles[*command] = style_index < imported.contour_styles.size() ?
			imported.contour_styles[style_index] : shape_style;
		++style_index;
	}
	if (!imported.contour_styles.empty())
		shape_style = imported.contour_styles.front();
	m_frame->shape_style = shape_style;
	ApplyShapeStyle();

	RefreshUndocmds();
}

wxString ASSDrawCanvas::GenerateDrawingASS()
{
	return ASSDrawEngine::GenerateASS();
}

wxString ASSDrawCanvas::GenerateASS()
{
	const std::vector<StyledSubshape> subshapes = BuildStyledSubshapes();
	std::vector<StyledAssSubshape> serializable;
	serializable.reserve(subshapes.size());
	for (std::vector<StyledSubshape>::const_iterator shape = subshapes.begin(); shape != subshapes.end(); ++shape)
	{
		StyledAssSubshape encoded;
		encoded.style = shape->style;
		encoded.commands.reserve(shape->commands.size());
		for (std::vector<DrawCmd*>::const_iterator command = shape->commands.begin(); command != shape->commands.end(); ++command)
			encoded.commands.push_back(ToAssDrawingCommand(*command));
		serializable.push_back(encoded);
	}
	const std::string output = SerializeStyledAss(serializable);
	return wxString(output.c_str(), wxConvUTF8);
}

DrawCmd* ASSDrawCanvas::AppendCmd(DrawCmd* cmd)
{
	DrawCmd* appended = ASSDrawEngine::AppendCmd(cmd);
	if (appended && appended->type == M)
		subshape_styles[appended] = shape_style;
	return appended;
}

void ASSDrawCanvas::InsertCmd(DrawCmd* cmd, DrawCmd* after)
{
	ASSDrawEngine::InsertCmd(cmd, after);
	if (cmd && cmd->type == M)
		subshape_styles[cmd] = shape_style;
}

bool ASSDrawCanvas::DeleteCommand(DrawCmd* cmd)
{
	const bool deleted = ASSDrawEngine::DeleteCommand(cmd);
	if (deleted)
	{
		subshape_styles.erase(cmd);
		if (coloring_target_shape == cmd)
		{
			coloring_target_shape = NULL;
			coloring_target_selected = false;
		}
	}
	return deleted;
}

void ASSDrawCanvas::SetShapeStyle(const ShapeStyle& new_style)
{
	shape_style = new_style;
	if (draw_mode == MODE_COLOR && coloring_target_shape)
		SetSubshapeStyle(coloring_target_shape, new_style);
	else
		for (std::map<DrawCmd*, ShapeStyle>::iterator style = subshape_styles.begin(); style != subshape_styles.end(); ++style)
			style->second = new_style;
	ApplyShapeStyle();
	RefreshDisplay();
}

void ASSDrawCanvas::ApplyShapeStyle()
{
	// These colors are kept separate from editor guides, points, and handles.
	rgba_shape_normal = ToAggColor(shape_style.fill_color,
		shape_style.fill_enabled ? shape_style.fill_opacity : 0);
	rgba_shape = rgba_shape_normal;
}

void ASSDrawCanvas::UpdateTranslations()
{
	bgimg.alpha_dlg->SetTitle(_("Background image opacity"));
	RefreshDisplay();
}

void ASSDrawCanvas::ResetEngine(bool addM)
{
	ClearPointsSelection();
	SetHighlighted(NULL, NULL);
	SetPreviewMode(false);
	SetDrawMode(MODE_ARR);
	subshape_styles.clear();
	coloring_target_shape = NULL;
	coloring_target_selected = false;
	ASSDrawEngine::ResetEngine(addM);
	RefreshUndocmds();
}

void ASSDrawCanvas::SetPreviewMode(bool mode)
{
	preview_mode = mode;
	if (preview_mode)
	{
		if (mousedownAt_point != NULL)
		{
			mousedownAt_point->cmd_main->Init();
			mousedownAt_point = NULL;
		}

		if (pointedAt_point != NULL)
			pointedAt_point = NULL;

		SetHighlighted( NULL, NULL );

		RefreshDisplay();
	}
}

// (Re)draw canvas
void ASSDrawCanvas::RefreshDisplay()
{
	ASSDrawEngine::RefreshDisplay();
	wxString asscmds = GenerateASS();
	if (oldasscmds != asscmds)
	{
		m_frame->UpdateASSCommandStringToSrcTxtCtrl(asscmds);
		oldasscmds = asscmds;
	}
}

void ASSDrawCanvas::SetDrawMode( MODE mode )
{
	draw_mode = mode;
	if (draw_mode != MODE_COLOR)
	{
		coloring_target_selected = false;
		coloring_target_shape = NULL;
	}

	if (!selected_points.empty())
		ClearPointsSelection();

	RefreshDisplay();

	if (IsTransformMode())
	{
		isshapetransformable = cmds.size() > 1;

		if (isshapetransformable)
		{

			// backup cmds
			backupcmds.free_all();
			for (DrawCmdList::iterator iterate = cmds.begin(); iterate != cmds.end(); iterate++)
			{
				DrawCmd* cmd = (*iterate);
				for (PointList::iterator iterate2 = cmd->controlpoints.begin(); iterate2 != cmd->controlpoints.end(); iterate2++)
				{
					wxPoint pp = (*iterate2)->ToWxPoint();
					backupcmds.move_to(pp.x, pp.y);
				}
				wxPoint pp = (*iterate)->m_point->ToWxPoint();
				backupcmds.move_to(pp.x, pp.y);
			}

			// calculate bounding rectangle
			agg::trans_affine mtx;
			trans_path *rm, *rb;
			agg::conv_curve<trans_path> *rc;
			ConstructPathsAndCurves(mtx, rm, rb, rc);
			rasterizer.reset();
			rasterizer.add_path(*rc);
			delete rm;
			delete rb;
			delete rc;
		    int minx = rasterizer.min_x(), miny = rasterizer.min_y();
		    int maxx = rasterizer.max_x(), maxy = rasterizer.max_y();

		    rectbound[0] = wxRealPoint(minx, miny);
		    rectbound[1] = wxRealPoint(maxx, miny);
		    rectbound[2] = wxRealPoint(maxx, maxy);
		    rectbound[3] = wxRealPoint(minx, maxy);
		    for (int i = 0; i < 4; i++)
		    	rectbound2[i] = rectbound[i];

		    rectbound2upd = -1;
		    rectbound2upd2 = -1;

		    backupowner = NONE;
		    InitiateDraggingIfTransformMode();

		    if (maxx - minx < 5 || maxy - miny < 5)
		    	isshapetransformable = false;
		}
	}

	RefreshUndocmds();
	m_frame->UpdateFrameUI();

}

void ASSDrawCanvas::SetDragMode( DRAGMODE mode )
{
	drag_mode = mode;
}

bool ASSDrawCanvas::IsTransformMode()
{
	return draw_mode == MODE_NUT_BILINEAR || draw_mode == MODE_SCALEROTATE;
}

bool ASSDrawCanvas::CanZoom()
{
	return !IsTransformMode() || !drag_mode.drawing;
}

bool ASSDrawCanvas::CanMove()
{
	return !IsTransformMode() || dragAnchor_left == NULL;
}

Point* ASSDrawCanvas::FindPointAtScreenPosition(const wxPoint& position, bool control_point) const
{
	const double hit_radius = kEditorPointHitRadius * std::max(1.0, GetDPIScaleFactor());
	const double maximum_distance_squared = hit_radius * hit_radius;
	double closest_distance_squared = maximum_distance_squared;
	Point* closest_point = NULL;

	for (DrawCmdList::const_iterator command = cmds.begin(); command != cmds.end(); ++command)
	{
		if (!control_point)
		{
			wxPoint candidate_position = (*command)->m_point->ToWxPoint();
			const double dx = candidate_position.x - position.x;
			const double dy = candidate_position.y - position.y;
			const double distance_squared = dx * dx + dy * dy;
			if (distance_squared <= closest_distance_squared)
			{
				closest_distance_squared = distance_squared;
				closest_point = (*command)->m_point;
			}
			continue;
		}

		for (PointList::const_iterator point = (*command)->controlpoints.begin();
			point != (*command)->controlpoints.end(); ++point)
		{
			wxPoint candidate_position = (*point)->ToWxPoint();
			const double dx = candidate_position.x - position.x;
			const double dy = candidate_position.y - position.y;
			const double distance_squared = dx * dx + dy * dy;
			if (distance_squared <= closest_distance_squared)
			{
				closest_distance_squared = distance_squared;
				closest_point = *point;
			}
		}
	}

	return closest_point;
}

bool ASSDrawCanvas::IsScreenPositionInFilledShape(const wxPoint& position) const
{
	if (pointsys->scale <= 0.0)
		return false;

	const wxRealPoint click((position.x - pointsys->originx) / pointsys->scale,
		(position.y - pointsys->originy) / pointsys->scale);
	int winding = 0;
	std::vector<wxRealPoint> contour;

	auto add_contour_winding = [&]() {
		if (contour.size() < 3)
			return;

		for (std::size_t index = 0; index < contour.size(); ++index)
		{
			const wxRealPoint& from = contour[index];
			const wxRealPoint& to = contour[(index + 1) % contour.size()];
			const double side = (to.x - from.x) * (click.y - from.y) -
				(to.y - from.y) * (click.x - from.x);
			if (from.y <= click.y)
			{
				if (to.y > click.y && side > 0.0)
					++winding;
			}
			else if (to.y <= click.y && side < 0.0)
			{
				--winding;
			}
		}
	};

	for (DrawCmdList::const_iterator command = cmds.begin(); command != cmds.end(); ++command)
	{
		DrawCmd* current = *command;
		if (current->type == M)
		{
			add_contour_winding();
			contour.clear();
			contour.push_back(wxRealPoint(current->m_point->x(), current->m_point->y()));
			continue;
		}

		if (contour.empty())
			continue;

		if (current->type == B && current->initialized && current->controlpoints.size() == 2)
		{
			PointList::const_iterator controls = current->controlpoints.begin();
			const wxRealPoint p0 = contour.back();
			const wxRealPoint p1((*controls)->x(), (*controls)->y());
			++controls;
			const wxRealPoint p2((*controls)->x(), (*controls)->y());
			const wxRealPoint p3(current->m_point->x(), current->m_point->y());
			for (int step = 1; step <= 16; ++step)
			{
				const double t = step / 16.0;
				const double inverse = 1.0 - t;
				contour.push_back(wxRealPoint(
					inverse * inverse * inverse * p0.x + 3.0 * inverse * inverse * t * p1.x +
					3.0 * inverse * t * t * p2.x + t * t * t * p3.x,
					inverse * inverse * inverse * p0.y + 3.0 * inverse * inverse * t * p1.y +
					3.0 * inverse * t * t * p2.y + t * t * t * p3.y));
			}
		}
		else
		{
			// Spline control points provide a conservative pick approximation.
			if (current->type == S)
			{
				for (PointList::const_iterator control = current->controlpoints.begin();
					control != current->controlpoints.end(); ++control)
					contour.push_back(wxRealPoint((*control)->x(), (*control)->y()));
			}
			contour.push_back(wxRealPoint(current->m_point->x(), current->m_point->y()));
		}
	}
	add_contour_winding();

	// AGG's path rasterizer uses the non-zero winding fill rule. An
	// oppositely-wound inner contour is therefore a hole and is not selectable.
	return winding != 0;
}

std::vector<StyledSubshape> ASSDrawCanvas::BuildStyledSubshapes() const
{
	struct RawContour
	{
		DrawCmd* start_command = NULL;
		std::vector<DrawCmd*> commands;
		std::vector<wxRealPoint> polygon;
		double area = 0.0;
		int parent = -1;
		bool is_hole = false;
	};

	std::vector<RawContour> contours;
	RawContour* current = NULL;
	for (DrawCmdList::const_iterator command = cmds.begin(); command != cmds.end(); ++command)
	{
		DrawCmd* draw_command = *command;
		if (draw_command->type == M)
		{
			contours.push_back(RawContour());
			current = &contours.back();
			current->start_command = draw_command;
			current->commands.push_back(draw_command);
			current->polygon.push_back(wxRealPoint(draw_command->m_point->x(), draw_command->m_point->y()));
			continue;
		}
		if (!current)
			continue;

		current->commands.push_back(draw_command);
		if (draw_command->type == B && draw_command->initialized && draw_command->controlpoints.size() == 2)
		{
			PointList::const_iterator controls = draw_command->controlpoints.begin();
			const wxRealPoint p0 = current->polygon.back();
			const wxRealPoint p1((*controls)->x(), (*controls)->y());
			++controls;
			const wxRealPoint p2((*controls)->x(), (*controls)->y());
			const wxRealPoint p3(draw_command->m_point->x(), draw_command->m_point->y());
			for (int step = 1; step <= 16; ++step)
			{
				const double t = step / 16.0;
				const double inverse = 1.0 - t;
				current->polygon.push_back(wxRealPoint(
					inverse * inverse * inverse * p0.x + 3.0 * inverse * inverse * t * p1.x + 3.0 * inverse * t * t * p2.x + t * t * t * p3.x,
					inverse * inverse * inverse * p0.y + 3.0 * inverse * inverse * t * p1.y + 3.0 * inverse * t * t * p2.y + t * t * t * p3.y));
			}
		}
		else
		{
			if (draw_command->type == S)
			{
				for (PointList::const_iterator control = draw_command->controlpoints.begin(); control != draw_command->controlpoints.end(); ++control)
					current->polygon.push_back(wxRealPoint((*control)->x(), (*control)->y()));
			}
			current->polygon.push_back(wxRealPoint(draw_command->m_point->x(), draw_command->m_point->y()));
		}
	}

	for (std::size_t index = 0; index < contours.size(); ++index)
	{
		contours[index].area = SignedArea(contours[index].polygon);
		if (contours[index].polygon.size() < 3)
			continue;
		for (std::size_t candidate = 0; candidate < contours.size(); ++candidate)
		{
			if (candidate == index || contours[candidate].polygon.size() < 3 ||
				!IsInsideContour(contours[candidate].polygon, contours[index].polygon.front()))
				continue;
			if (contours[index].parent == -1 || fabs(contours[candidate].area) < fabs(contours[contours[index].parent].area))
				contours[index].parent = static_cast<int>(candidate);
		}
		if (contours[index].parent >= 0)
		{
			const double parent_area = contours[contours[index].parent].area;
			contours[index].is_hole = contours[index].area * parent_area < 0.0;
		}
	}

	std::vector<StyledSubshape> result;
	for (std::size_t index = 0; index < contours.size(); ++index)
	{
		if (contours[index].is_hole)
			continue;
		StyledSubshape shape;
		shape.start_command = contours[index].start_command;
		shape.commands = contours[index].commands;
		shape.contours.push_back(contours[index].polygon);
		shape.style = GetSubshapeStyle(shape.start_command);
		for (std::size_t child = 0; child < contours.size(); ++child)
		{
			if (contours[child].is_hole && contours[child].parent == static_cast<int>(index))
			{
				shape.commands.insert(shape.commands.end(), contours[child].commands.begin(), contours[child].commands.end());
				shape.contours.push_back(contours[child].polygon);
			}
		}
		result.push_back(shape);
	}
	return result;
}

ShapeStyle ASSDrawCanvas::GetSubshapeStyle(DrawCmd* start_command) const
{
	std::map<DrawCmd*, ShapeStyle>::const_iterator found = subshape_styles.find(start_command);
	return found == subshape_styles.end() ? shape_style : found->second;
}

void ASSDrawCanvas::SetSubshapeStyle(DrawCmd* start_command, const ShapeStyle& style)
{
	if (!start_command)
		return;
	subshape_styles[start_command] = style;
	shape_style = style;
	m_frame->shape_style = style;
	ApplyShapeStyle();
	RefreshDisplay();
}

DrawCmd* ASSDrawCanvas::FindSubshapeAtScreenPosition(const wxPoint& position) const
{
	if (pointsys->scale <= 0.0)
		return NULL;
	const wxRealPoint click((position.x - pointsys->originx) / pointsys->scale,
		(position.y - pointsys->originy) / pointsys->scale);
	DrawCmd* selected = NULL;
	const std::vector<StyledSubshape> shapes = BuildStyledSubshapes();
	for (std::vector<StyledSubshape>::const_iterator shape = shapes.begin(); shape != shapes.end(); ++shape)
	{
		int winding = 0;
		for (std::vector< std::vector<wxRealPoint> >::const_iterator contour = shape->contours.begin(); contour != shape->contours.end(); ++contour)
			winding += WindingAtPoint(*contour, click);
		if (winding != 0)
			selected = shape->start_command;
	}
	return selected;
}

void ASSDrawCanvas::SelectColoringTarget(const wxPoint& position)
{
	coloring_target_shape = FindSubshapeAtScreenPosition(position);
	coloring_target_selected = coloring_target_shape != NULL;
	if (coloring_target_selected)
		m_frame->SetStatusText(_("Shape selected. Double-click to choose its fill color."), 1);
	else
		m_frame->SetStatusText(_("Click a filled part of the shape (not a hole)."), 1);
	RefreshDisplay();
}

void ASSDrawCanvas::ShowColorSelector()
{
	if (!coloring_target_shape)
		return;
	const ShapeStyle selected_style = GetSubshapeStyle(coloring_target_shape);
	wxColourData color_data;
	color_data.SetChooseFull(true);
	color_data.SetColour(wxColour(selected_style.fill_color.red, selected_style.fill_color.green,
		selected_style.fill_color.blue));
	wxColourDialog dialog(m_frame, &color_data);
	dialog.SetTitle(_("Choose fill color"));
	if (dialog.ShowModal() != wxID_OK)
		return;

	const wxColour color = dialog.GetColourData().GetColour();
	AddUndo(wxTRANSLATE("Change sub-shape color"));
	ShapeStyle style = selected_style;
	style.fill_enabled = true;
	style.fill_color = { color.Red(), color.Green(), color.Blue() };
	SetSubshapeStyle(coloring_target_shape, style);
	RefreshUndocmds();
}

// Do the dragging
void ASSDrawCanvas::OnMouseMove(wxMouseEvent &event)
{
	mouse_point = event.GetPosition();
	int xx, yy, wx, wy;
	xx = mouse_point.x; yy = mouse_point.y;
	pointsys->FromWxPoint ( mouse_point, wx, wy );
	if (event.Dragging())
	{
		if (IsTransformMode() && isshapetransformable && backupowner == LEFT)
		{
			// update bounding polygon
			if (rectbound2upd > -1)
			{
				if (!dragAnchor_left) dragAnchor_left = new wxPoint(mouse_point);
				wxPoint diff = mouse_point - *dragAnchor_left;
				wxRealPoint rdiff(diff.x, diff.y);
				switch(draw_mode)
				{
				case MODE_NUT_BILINEAR:
					if (rectbound2upd2 == -1) //only one vertex dragged
						rectbound2[rectbound2upd].x = xx, rectbound2[rectbound2upd].y = yy;
					else
					{
						rectbound2[rectbound2upd].x += diff.x, rectbound2[rectbound2upd].y += diff.y;
						rectbound2[rectbound2upd2].x += diff.x, rectbound2[rectbound2upd2].y += diff.y;
					}
					undodesc = wxTRANSLATE("Bilinear transform");
					*dragAnchor_left = mouse_point;
					break;
				case MODE_SCALEROTATE:
					if (rectbound2upd2 == -1) //only one vertex dragged
					{
						int adjacent[2] = { (rectbound2upd + 3) % 4, (rectbound2upd + 1) % 4 };
						int opposite = (rectbound2upd + 2) % 4;
						wxRealPoint newpoint = backup[rectbound2upd] + rdiff;
						double nx, ny;
						for (int i = 0; i < 2; i++)
						{
							bool isect = agg::calc_intersection(
								backup[opposite].x, backup[opposite].y,
								backup[adjacent[i]].x, backup[adjacent[i]].y,
								newpoint.x, newpoint.y,
								backup[adjacent[i]].x + diff.x, backup[adjacent[i]].y + diff.y,
								&nx, &ny);
							if (isect && !(fabs(nx - backup[opposite].x) < 10 && fabs(ny - backup[opposite].y) < 10))
								rectbound2[adjacent[i]] = wxRealPoint(nx, ny);
						}
						GetThe4thPoint(backup[opposite].x, backup[opposite].y,
							rectbound2[adjacent[0]].x, rectbound2[adjacent[0]].y,
							rectbound2[adjacent[1]].x, rectbound2[adjacent[1]].y,
							&rectbound2[rectbound2upd].x, &rectbound2[rectbound2upd].y);
						if (event.ShiftDown()) // shift down = maintain aspect ratio (damn so complicated)
						{
							// first create the guide points, which are basically backup points reoriented based on mouse position
							wxRealPoint guide[4];
							guide[opposite] = backup[opposite];
							guide[adjacent[0]] = backup[adjacent[0]];
							guide[adjacent[1]] = backup[adjacent[1]];
							for (int i = 0; i < 2; i++)
							{
								if ((rectbound2[adjacent[i]].x < guide[opposite].x && guide[opposite].x < guide[adjacent[i]].x)
									|| (rectbound2[adjacent[i]].x > guide[opposite].x && guide[opposite].x > guide[adjacent[i]].x)
									|| (rectbound2[adjacent[i]].y < guide[opposite].y && guide[opposite].y < guide[adjacent[i]].y)
									|| (rectbound2[adjacent[i]].y > guide[opposite].y && guide[opposite].y > guide[adjacent[i]].y))
								{
									guide[adjacent[i]] = guide[opposite] - (guide[adjacent[i]] - guide[opposite]);
								}
							}
							guide[rectbound2upd] = guide[adjacent[0]] + (guide[adjacent[1]] - guide[opposite]);
							// now we determine which rescaled sides have larger enlargement/shrinkage ratio ..
							double ix[2], iy[2], dx[2], dy[2];
							for (int i = 0; i < 2; i++)
							{
								agg::calc_intersection(guide[opposite].x, guide[opposite].y, guide[rectbound2upd].x, guide[rectbound2upd].y,
									rectbound2[rectbound2upd].x, rectbound2[rectbound2upd].y,
									rectbound2[adjacent[i]].x, rectbound2[adjacent[i]].y, &ix[i], &iy[i]);
								dx[i] = ix[i] - guide[opposite].x;
								dy[i] = iy[i] - guide[opposite].y;
							}
							// .. and modify the other sides to follow the ratio
							for (int i = 0; i < 2; i++)
							{
								int j = (i + 1) % 2;
								if (fabs(dx[i]) > fabs(dx[j]) || fabs(dy[i]) > fabs(dy[j]))
								{
									rectbound2[rectbound2upd].x = ix[i];
									rectbound2[rectbound2upd].y = iy[i];
									GetThe4thPoint(rectbound2[adjacent[i]].x, rectbound2[adjacent[i]].y,
										guide[opposite].x, guide[opposite].y, ix[i], iy[i],
										&rectbound2[adjacent[j]].x, &rectbound2[adjacent[j]].y);
								}
							}
						}
					}
					else // an edge dragged (i.e. move 2 vertices in sync)
					{
						// it is guaranteed that rectbound2upd and rectbound2upd2 are in clockwise direction
						// from the way they are detected in OnMouseLeftDown()
						int toupd[2] = { rectbound2upd, rectbound2upd2 };
						int adjacent[2] = { (rectbound2upd2 + 2) % 4, (rectbound2upd2 + 1) % 4 };
						wxRealPoint vertexone = backup[toupd[0]] + rdiff, vertextwo = backup[toupd[1]] + rdiff;
						double nx, ny;
						for (int i = 0; i < 2; i++)
						{
							agg::calc_intersection(
								rectbound2[adjacent[i]].x, rectbound2[adjacent[i]].y,
								backup[toupd[i]].x, backup[toupd[i]].y,
								vertexone.x, vertexone.y,
								vertextwo.x, vertextwo.y,
								&nx, &ny);
							if (!(fabs(nx - backup[adjacent[i]].x) < 10 && fabs(ny - backup[adjacent[i]].y) < 10))
								rectbound2[toupd[i]].x = (int) nx, rectbound2[toupd[i]].y = (int) ny;
						}
					}
					UpdateTranformModeRectCenter();
					undodesc = wxTRANSLATE("Scale");
					break;
				}

				// update points
				UpdateNonUniformTransformation();
				RefreshDisplay();
			}
		}
		else if (draw_mode != MODE_DEL)
		{

			// point left-dragged
			if (mousedownAt_point != NULL && mousedownAt_point->isselected)
			{
				if (!mousedownAt_point->IsAt( wx, wy ))
				{
					if (draw_mode == MODE_ARR) {
						int movex = wx - mousedownAt_point->x(), movey = wy - mousedownAt_point->y();
						PointSet::iterator iter = selected_points.begin();
						for (; iter != selected_points.end(); iter++)
							(*iter)->setXY( (*iter)->x() + movex, (*iter)->y() + movey );
					}
					else
						mousedownAt_point->setXY( wx, wy );

					EnforceC1Continuity (mousedownAt_point->cmd_main, mousedownAt_point);

					RefreshDisplay();
					if (undodesc == _T(""))
					{
						if (mousedownAt_point->type == MP)
							undodesc = wxTRANSLATE("Drag point");
						else
							undodesc = wxTRANSLATE("Drag control point");
					}
				}
			}
			// origin left-dragged
			else if (dragOrigin)
			{
				if (wx != 0 || wy != 0)
				{
					wxPoint wxp = pointsys->ToWxPoint ( wx, wy );
					MovePoints(-wx, -wy);
					pointsys->originx = wxp.x;
					pointsys->originy = wxp.y;
					RefreshDisplay();
					undodesc = wxTRANSLATE("Move origin");
				}
			}
			else if (dragAnchor_left != NULL)
			{
				// handle left-dragging here
				if (lastDrag_left && dragAnchor_left != lastDrag_left)
			       delete lastDrag_left;
				lastDrag_left = new wxPoint(xx, yy);
				int ax = dragAnchor_left->x, ay = dragAnchor_left->y;
				int sx = lastDrag_left->x, sy = lastDrag_left->y;
				int lx, rx, ty, by;
				if (ax > sx) lx = sx, rx = ax;
				else lx = ax, rx = sx;
				if (ay > sy) ty = sy, by = ay;
				else ty = ay, by = sy;
				SelectPointsWithin( lx, rx, ty, by, GetSelectMode(event) );
				RefreshDisplay();
			}
		}

		// right-dragged
		if (dragAnchor_right != NULL)
		{
			if (draw_mode == MODE_SCALEROTATE)
			{
				if (backupowner == RIGHT)
				{
					double cx = rectcenter.x, cy = rectcenter.y; // center x,y
					double diameter = sqrt(pow(rectbound2[0].x - rectbound2[2].x, 2) + pow(rectbound2[0].y - rectbound2[2].y, 2));
					double radius = diameter / 2;
					double old_dx = dragAnchor_right->x - cx, old_dy = dragAnchor_right->y - cy;
					double new_dx = mouse_point.x - cx, new_dy = mouse_point.y - cy;
					double old_angle = atan2(old_dy, old_dx);
					double new_angle = atan2(new_dy, new_dx);
					double delta = new_angle - old_angle;
					for (int i = 0; i < 4; i++)
					{
						old_dx = backup[i].x - cx, old_dy = backup[i].y - cy;
						old_angle = atan2(old_dy, old_dx);
						new_angle = old_angle + delta;
						new_dx = radius * cos(new_angle), new_dy = radius * sin(new_angle);
						rectbound2[i].x = (int)(new_dx + cx), rectbound2[i].y = (int)(new_dy + cy);
					}
					UpdateNonUniformTransformation();
					RefreshDisplay();
					undodesc = wxTRANSLATE("Rotate");
				}
			}
			else if (CanMove())
			{
				MoveCanvas(xx - dragAnchor_right->x, yy - dragAnchor_right->y);
				dragAnchor_right->x = xx;
				dragAnchor_right->y = yy;
				RefreshDisplay();
			}
		}
	}
	else if (!preview_mode)// not dragging and not preview mode
    {
		if (IsTransformMode())
		{
			const double transform_hit_radius = kTransformHandleRadius * std::max(1.0, GetDPIScaleFactor());
			int oldrectbound = rectbound2upd;
			rectbound2upd = -1;
			rectbound2upd2 = -1;
			for (int i = 0; i < 4; i++)
			{
				if (abs((int)rectbound2[i].x - mouse_point.x) <= transform_hit_radius
					&& abs((int)rectbound2[i].y - mouse_point.y) <= transform_hit_radius)
						rectbound2upd = i;
			}
			for (int i = 0; rectbound2upd == -1 && i < 4; i++)
			{
				int j = (i+1) % 4;
				wxRealPoint &pi = rectbound2[i], &pj = rectbound2[j];
				double dy = fabs(pj.y - pi.y);
				double dy3 = dy / 3;
				double dx = fabs(pj.x - pi.x);
				double dx3 = dx / 3;
				double ix, iy;
				bool intersect = false;
				if (dy > dx)
				{
					intersect = agg::calc_intersection(
						pi.x, pi.y, pj.x, pj.y,
						mouse_point.x - transform_hit_radius, mouse_point.y,
						mouse_point.x + transform_hit_radius, mouse_point.y, &ix, &iy);
					intersect &= fabs(mouse_point.x - ix) <= transform_hit_radius;
					intersect &= (pj.y > pi.y?
						pj.y - dy3 > iy && iy > pi.y + dy3:
						pj.y + dy3 < iy && iy < pi.y - dy3);
				}
				else
				{
					intersect = agg::calc_intersection(
						pi.x, pi.y, pj.x, pj.y,
						mouse_point.x, mouse_point.y - transform_hit_radius,
						mouse_point.x, mouse_point.y + transform_hit_radius, &ix, &iy);
					intersect &= fabs(mouse_point.y - iy) <= transform_hit_radius;
					intersect &= (pj.x > pi.x?
						pj.x - dx3 > ix && ix > pi.x + dx3:
						pj.x + dx3 < ix && ix < pi.x - dx3);
				}
				if (intersect)
				{
					rectbound2upd = i;
					rectbound2upd2 = j;
				}
			}
			if (rectbound2upd != -1 || oldrectbound != -1)
				RefreshDisplay();
		}
		else
		{
	         /* figure out if the mouse is pointing at a point of a command
	            we need not do this for dragging since this same block has set
	            the related variables before the user starts to hold the button
	            (well, before you can drag something you have to move the pointer
	            over that thing first, right?) and we don't want to mess with those
	            when it's dragging
	         */

			// Check controls first, then primary points.  Primary points keep the
			// historical priority where they overlap a Bezier handle.
			Point* last_pointedAt_point = pointedAt_point;
			pointedAt_point = FindPointAtScreenPosition(mouse_point, true);
			Point* main_point = FindPointAtScreenPosition(mouse_point, false);
			if (main_point != NULL)
				pointedAt_point = main_point;

			if (pointedAt_point != last_pointedAt_point)
			{
				if (pointedAt_point != NULL)
					SetHighlighted( pointedAt_point->cmd_main, pointedAt_point );
				else
					SetHighlighted( NULL, NULL );
				RefreshDisplay();
			}
		}
	} // not dragging and preview mode = ignore all mouse movements

	// we are not done yet?
    // oh yeah, we need to set the status bar just for fun
    if (hasStatusBar)
	{
        m_frame->SetStatusText(
            wxString::Format( _T("%5d %5d"), (int)wx, (int)wy ), 0 );
        if (pointedAt_point == NULL ||
             (newcommand != NULL && !newcommand->initialized) )
           m_frame->SetStatusText( _T(""), 1 );
        else
           m_frame->SetStatusText( _T(" ") + pointedAt_point->cmd_main->ToString().Upper(), 1 );
	}

}

// End drag points
void ASSDrawCanvas::OnMouseLeftUp(wxMouseEvent& event)
{
	ProcessOnMouseLeftUp();
	event.Skip( true );
}

void ASSDrawCanvas::OnMouseLeftDClick(wxMouseEvent& event)
{
	if (draw_mode != MODE_COLOR)
	{
		event.Skip();
		return;
	}

	SelectColoringTarget(event.GetPosition());
	if (coloring_target_selected)
		ShowColorSelector();
}

void ASSDrawCanvas::ProcessOnMouseLeftUp()
{
	if (!capturemouse_left) return;

	// draw mode
	if (newcommand != NULL)
	{
		newcommand->Init();
		switch (newcommand->type)
		{
		case M:
			undodesc = wxTRANSLATE("Add a new M"); break;
		case L:
			undodesc = wxTRANSLATE("Add a new L"); break;
		case B:
			undodesc = wxTRANSLATE("Add a new B"); break;
		}
		newcommand = NULL;
		// we need to manually refresh the GUI to draw the new control points
		RefreshDisplay();
	}
	else if ( draw_mode == MODE_DEL // if it's delete mode
				&& mousedownAt_point != NULL // and mouse down at a point
				&& mousedownAt_point == pointedAt_point ) // and released at the same point
	{
		// first take care of mouse readings
		pointedAt_point = NULL;
		Point *lastmousedownAt_point = mousedownAt_point;
		mousedownAt_point = NULL;

		// try deleting
		CMDTYPE t = lastmousedownAt_point->cmd_main->type;
		if ( DeleteCommand( lastmousedownAt_point->cmd_main ) )
		{
			ClearPointsSelection();
			SetHighlighted( NULL, NULL );
			RefreshDisplay();
			switch (t)
			{
			case M:
				undodesc = wxTRANSLATE("Delete a M"); break;
			case L:
				undodesc = wxTRANSLATE("Delete a L"); break;
			case B:
				undodesc = wxTRANSLATE("Delete a B"); break;
			}
		}
		else
		{
			RefreshDisplay(); // redraw before showing the error box
			wxMessageDialog msgb(m_frame,
			_("You must delete that command/point last"),
			_("Error"), wxOK | wxICON_EXCLAMATION );
			msgb.ShowModal();
		}
	}
	else if ( lastDrag_left && dragAnchor_left ) // point selection
	{
		if (lastDrag_left && dragAnchor_left != lastDrag_left)
		    delete lastDrag_left;
	    delete dragAnchor_left;
		lastDrag_left = NULL;
		dragAnchor_left = NULL;
	}

	if (dragOrigin)
	{
		dragOrigin = false;
		RefreshDisplay(); // clear the crosshair
	}

	rectbound2upd = -1;
	rectbound2upd2 = -1;
    backupowner = NONE;

	if (!undodesc.IsSameAs(_T("")))
	{
		AddUndo( undodesc );
		undodesc = _T("");
		RefreshUndocmds();
	}

	mousedownAt_point = NULL;

	if (HasCapture())
		ReleaseMouse();
	capturemouse_left = false;

	RefreshDisplay();
}

// Set up to drag points, if any
void ASSDrawCanvas::OnMouseLeftDown(wxMouseEvent& event)
{

    // no drawing in preview mode
	if (preview_mode)
		return;

	wxPoint q = event.GetPosition();
	if (draw_mode == MODE_COLOR)
	{
		SelectColoringTarget(q);
		return;
	}

	// wxPoint to Point
	int px, py;
	pointsys->FromWxPoint(q, px, py);

	// A click may arrive before a motion event (for example after changing
	// windows), so refresh the device-space hit target here as well.
	if (!IsTransformMode())
	{
		pointedAt_point = FindPointAtScreenPosition(q, true);
		Point* main_point = FindPointAtScreenPosition(q, false);
		if (main_point != NULL)
			pointedAt_point = main_point;
	}

	// create new cmd if in draw mode / or delete point if del tool selected
	switch (draw_mode)
	{
	case MODE_M:
		newcommand = NewCmd(M, px, py);
		break;
	case MODE_L:
		newcommand = NewCmd(L, px, py);
		break;
	case MODE_B:
		newcommand = NewCmd(B, px, py);
		break;
	}

	// continue working on the new command (if any)
	// only if it is not mouse down on any control point
	if (newcommand != NULL)
	{
		if (pointedAt_point != NULL && pointedAt_point->type == CP)
		{
			// oops, user clicked on a control point so cancel new command
			// and let him drag the control point
			delete newcommand;
            newcommand = NULL;
		}
		else
		{
			// first set to drag the new command no matter what
			mousedownAt_point = newcommand->m_point;

			// if user drags from existing point, insert the new command
			// else append the new command
			if (pointedAt_point != NULL)
				InsertCmd( newcommand, pointedAt_point->cmd_main );
			else
			{
				if (cmds.empty() && newcommand->type != M)
					AppendCmd( M, px, py );
				newcommand = AppendCmd( newcommand );
			}

			pointedAt_point = mousedownAt_point;

			//highlight it
			SetHighlighted( newcommand, newcommand->m_point );
		}
	}

	// we already calculated pointedAt_point in OnMouseMove() so just use it
	mousedownAt_point = pointedAt_point;
	SELECTMODE smode = GetSelectMode(event);
	if (mousedownAt_point && !mousedownAt_point->isselected)
	{
		if (smode == NEW)
		{
			ClearPointsSelection();
			mousedownAt_point->isselected = true;
			selected_points.insert(mousedownAt_point);
		}
		else
		{
			wxPoint wxp = mousedownAt_point->ToWxPoint();
			SelectPointsWithin(wxp.x, wxp.x, wxp.y, wxp.y, smode);
		}
	}
	else if (!mousedownAt_point && smode == NEW)
		ClearPointsSelection();

	if ( mousedownAt_point == NULL && px == 0 && py == 0 )
		dragOrigin = true;

	if ( mousedownAt_point == NULL && !dragOrigin )
	{
		dragAnchor_left = new wxPoint(q.x, q.y);
		lastDrag_left = dragAnchor_left;
	}

	if (InitiateDraggingIfTransformMode())
	    backupowner = LEFT;

	CaptureMouse();
	capturemouse_left = true;
	RefreshDisplay();

	event.Skip( true );
}

// End drag the canvas
void ASSDrawCanvas::OnMouseRightUp(wxMouseEvent& event)
{
	ProcessOnMouseRightUp();
	event.Skip( true );
}

void ASSDrawCanvas::ProcessOnMouseRightUp()
{
	if (!capturemouse_right) return;

    if (lastDrag_right && dragAnchor_right != lastDrag_right)
        delete lastDrag_right;
    if (dragAnchor_right)
        delete dragAnchor_right;
	dragAnchor_right = NULL;
	lastDrag_right = NULL;

	// don't crash the program
	if (HasCapture())
		ReleaseMouse();
	capturemouse_right = false;

	rectbound2upd = -1;
	rectbound2upd2 = -1;
    backupowner = NONE;

	if (!undodesc.IsSameAs(_T("")))
	{
		AddUndo( undodesc );
		undodesc = _T("");
		RefreshUndocmds();
	}

	RefreshDisplay();
	SetFocus();
}

// Set up to drag the canvas
void ASSDrawCanvas::OnMouseRightDown(wxMouseEvent &event)
{
	wxPoint q = event.GetPosition();
	dragAnchor_right = new wxPoint(q.x, q.y);
	lastDrag_right = dragAnchor_right;
	CaptureMouse();
	capturemouse_right = true;

	if (InitiateDraggingIfTransformMode())
	    backupowner = RIGHT;

	event.Skip( true );
}

// Reset to point-and-drag mode
void ASSDrawCanvas::OnMouseRightDClick(wxMouseEvent& event)
{
	wxMenu* menu = new wxMenu();

	if (pointedAt_point)
	{
		ProcessOnMouseLeftUp();
		ProcessOnMouseRightUp();
		dblclicked_point_right = pointedAt_point;
		pointedAt_point = NULL;
		wxMenuItem *cmdmenuitem = new wxMenuItem(menu, MENU_DUMMY, dblclicked_point_right->cmd_main->ToString());
#ifdef __WINDOWS__
		wxFont f = cmdmenuitem->GetFont();
		f.SetWeight(wxFONTWEIGHT_BOLD);
		cmdmenuitem->SetFont(f);
#endif
		menu->Append(cmdmenuitem);
		menu->Enable(MENU_DUMMY, false);
		switch (dblclicked_point_right->cmd_main->type)
		{
			case L:
				menu->Append(MENU_DRC_LNTOBEZ, _("Convert to Bezier curve (B command)"));
				break;
			case B:
				if (dblclicked_point_right->type != MP) break;
				menu->AppendCheckItem(MENU_DRC_BEZTOLN, _("Convert to line (L command)"));
				if (dblclicked_point_right->cmd_next && dblclicked_point_right->cmd_next->type == B)
				{
					menu->AppendCheckItem(MENU_DRC_C1CONTBEZ, _("Smooth connection"));
					if (static_cast<DrawCmd_B*>(dblclicked_point_right->cmd_main)->C1Cont)
						menu->Check(MENU_DRC_C1CONTBEZ, true);
				}
				break;
		}

	}
	else
	{
		menu->Append(MENU_DRC_MOVE00, _("Move [0,0] here"));
		menu->AppendSeparator();
		menu->AppendRadioItem(MODE_ARR, _("Mode: D&rag"));
		menu->AppendRadioItem(MODE_M, _("Mode: Draw &M"));
		menu->AppendRadioItem(MODE_L, _("Mode: Draw &L"));
		menu->AppendRadioItem(MODE_B, _("Mode: Draw &B"));
		menu->AppendRadioItem(MODE_DEL, _("Mode: &Delete"));
		menu->AppendRadioItem(MODE_SCALEROTATE, _("Mode: Scale/Rotate"));
		menu->AppendRadioItem(MODE_NUT_BILINEAR, _("Mode: Bilinear transformation"));
		menu->AppendRadioItem(MODE_COLOR, _("Mode: Coloring"));
		menu->Check(GetDrawMode(), true);
	}

	if (menu->GetMenuItemCount() > 0) // only if there is menu item
	{
		SetHighlighted(NULL, NULL);
		mousedownAt_point = NULL;
		RefreshDisplay();
		AssdrawTheme::ApplyMenu(menu);
		PopupMenu(menu);
	}
	delete menu;

	event.Skip( true );
}

bool ASSDrawCanvas::InitiateDraggingIfTransformMode()
{
	if (IsTransformMode() && isshapetransformable && backupowner == NONE)
	{
		for (int i = 0; i < 4; i++)
			backup[i] = rectbound2[i];
		UpdateTranformModeRectCenter();
		return true;
	}
	else
		return false;
}

void ASSDrawCanvas::UpdateTranformModeRectCenter()
{
	double cx, cy;
	if (agg::calc_intersection(rectbound2[0].x, rectbound2[0].y, rectbound2[2].x, rectbound2[2].y,
		rectbound2[1].x, rectbound2[1].y, rectbound2[3].x, rectbound2[3].y, &cx, &cy))
	{
		rectcenter = wxRealPoint(cx, cy);
	}
}

bool ASSDrawCanvas::GetThe4thPoint(double ox, double oy, double a1x, double a1y, double a2x, double a2y, double *x, double *y)
{
	*x = a1x + a2x - ox;
	*y = a1y + a2y - oy;
	return true;
}

// Mousewheel
void ASSDrawCanvas::OnMouseWheel(wxMouseEvent &event)
{
	double amount = event.GetWheelRotation() / event.GetWheelDelta();
	if (event.ControlDown()) amount /= 10.0;
	bgimg.new_center = wxRealPoint(mouse_point.x, mouse_point.y);
	ChangeZoomLevel( amount, mouse_point );
}

void ASSDrawCanvas::OnDPIChanged(wxDPIChangedEvent &event)
{
	RefreshDisplay();
	event.Skip();
}

void ASSDrawCanvas::ChangeZoomLevelTo( double zoom, wxPoint bgzoomctr )
{
	ChangeZoomLevel(zoom - pointsys->scale, bgzoomctr);
}

void ASSDrawCanvas::ChangeZoomLevel( double amount, wxPoint bgzoomctr )
{
	double old_scale = pointsys->scale;
	if (CanZoom() && drag_mode.drawing)
		ChangeDrawingZoomLevel( amount );

	if (CanZoom() && drag_mode.bgimg)
		if (drag_mode.drawing)
		    ChangeBackgroundZoomLevel(bgimg.scale * pointsys->scale / old_scale, wxRealPoint(pointsys->originx, pointsys->originy));
		else
		    ChangeBackgroundZoomLevel(bgimg.scale + amount / 10.0,  wxRealPoint(bgzoomctr.x, bgzoomctr.y));

	RefreshDisplay();
}

void ASSDrawCanvas::ChangeDrawingZoomLevel( double scrollamount )
{
	if (!CanZoom()) return;
    double zoom = pointsys->scale + scrollamount;
    if (zoom <= 50.0)
	{
		if (zoom < 1.0) zoom = 1.0;
		pointsys->scale = zoom;
	}

	m_frame->UpdateFrameUI();
}

void ASSDrawCanvas::ChangeBackgroundZoomLevel(double zoom, wxRealPoint newcenter)
{
	bgimg.new_scale = zoom;
	bgimg.new_center = newcenter;
	if (bgimg.new_scale < 0.01) bgimg.new_scale = 0.01;
	UpdateBackgroundImgScalePosition();
}

void ASSDrawCanvas::MoveCanvasOriginTo(double originx, double originy)
{
	MoveCanvas(originx - pointsys->originx, originy - pointsys->originy);
}

void ASSDrawCanvas::MoveCanvas(double xamount, double yamount)
{
	if (CanMove() && drag_mode.drawing)
		MoveCanvasDrawing(xamount, yamount);
	if (CanMove() && drag_mode.bgimg)
		MoveCanvasBackground(xamount, yamount);
}

void ASSDrawCanvas::MoveCanvasDrawing(double xamount, double yamount)
{
	if (!CanMove()) return;
	pointsys->originx += xamount;
	pointsys->originy += yamount;
	if (IsTransformMode())
	{
	    for (int i = 0; i < 4; i++)
	    {
	    	rectbound[i].x += (int) xamount;
	    	rectbound[i].y += (int) yamount;
	    	rectbound2[i].x += (int) xamount;
	    	rectbound2[i].y += (int) yamount;
		}
		unsigned vertices = backupcmds.total_vertices();
		double x, y;
		for (int i = 0; i < vertices; i++)
		{
			backupcmds.vertex(i, &x, &y);
	        backupcmds.modify_vertex(i, x + xamount, y + yamount);
		}
	}
}

void ASSDrawCanvas::MoveCanvasBackground(double xamount, double yamount)
{
	bgimg.new_disp.x += xamount;
	bgimg.new_disp.y += yamount;
	UpdateBackgroundImgScalePosition();
}

void ASSDrawCanvas::OnSelect_ConvertLineToBezier(wxCommandEvent&)
{
	if (!dblclicked_point_right) return;
	AddUndo( wxTRANSLATE("Convert Line to Bezier") );
	DrawCmd_B *newB = new DrawCmd_B(dblclicked_point_right->x(), dblclicked_point_right->y(),
		pointsys, dblclicked_point_right->cmd_main);
	InsertCmd ( newB, dblclicked_point_right->cmd_main );
	ClearPointsSelection();
	SetHighlighted(NULL, NULL);
	DeleteCommand(dblclicked_point_right->cmd_main);
	pointedAt_point = newB->m_point;
	newB->Init();
	RefreshDisplay();
	RefreshUndocmds();
}

void ASSDrawCanvas::OnSelect_ConvertBezierToLine(wxCommandEvent&)
{
	if (!dblclicked_point_right) return;
	AddUndo( wxTRANSLATE("Convert Bezier to Line") );
	DrawCmd_L *newL = new DrawCmd_L(dblclicked_point_right->x(), dblclicked_point_right->y(), pointsys, dblclicked_point_right->cmd_main);
	InsertCmd ( newL, dblclicked_point_right->cmd_main );
	ClearPointsSelection();
	SetHighlighted(NULL, NULL);
	DeleteCommand(dblclicked_point_right->cmd_main);
	pointedAt_point = newL->m_point;
	newL->Init();
	RefreshDisplay();
	RefreshUndocmds();
}

void ASSDrawCanvas::OnSelect_C1ContinuityBezier(wxCommandEvent&)
{
	if (!dblclicked_point_right) return;
	DrawCmd_B *cmdb = static_cast<DrawCmd_B*>(dblclicked_point_right->cmd_main);
	AddUndo( cmdb->C1Cont? wxTRANSLATE("Unset continuous"):wxTRANSLATE("Set continuous") );
	cmdb->C1Cont = !cmdb->C1Cont;
	RefreshUndocmds();
}

void ASSDrawCanvas::OnSelect_Move00Here(wxCommandEvent&)
{
	AddUndo( wxTRANSLATE("Move origin") );
	int wx, wy;
	pointsys->FromWxPoint(mouse_point, wx, wy);
	wxPoint wxp = pointsys->ToWxPoint(wx, wy);
	pointsys->originx = wxp.x;
	pointsys->originy = wxp.y;
	MovePoints(-wx, -wy);
	RefreshDisplay();
	RefreshUndocmds();
}

void ASSDrawCanvas::ConnectSubsequentCmds (DrawCmd* cmd1, DrawCmd* cmd2)
{
	ASSDrawEngine::ConnectSubsequentCmds(cmd1, cmd2);
	if (cmd1 && cmd1->type == B)
	{
		DrawCmd_B *cmd1b = static_cast< DrawCmd_B* >(cmd1);
		cmd1b->C1Cont = false;
	}
}

void ASSDrawCanvas::EnforceC1Continuity (DrawCmd* cmd, Point* pnt)
{
	if (!cmd || !pnt) return;
	if (cmd->type != B && cmd->type != S) return;
	if (pnt->type == MP) return;
	Point *theotherpoint, *mainpoint;
	if (pnt->num == 1)
	{
		if (!cmd->prev || (cmd->prev->type != B && cmd->prev->type != S)) return;
		DrawCmd_B *prevb = static_cast< DrawCmd_B* >(cmd->prev);
		if (!prevb->C1Cont) return;
		PointList::iterator it = prevb->controlpoints.end();
		it--;
		theotherpoint = *it;
		mainpoint = prevb->m_point;
	}
	else if (pnt->num == cmd->controlpoints.size())
	{
		DrawCmd_B *thisb = static_cast< DrawCmd_B* >(cmd);
		if (!thisb->C1Cont) return;
		theotherpoint = *(cmd->m_point->cmd_next->controlpoints.begin());
		mainpoint = thisb->m_point;
	}
	else
		return;
	theotherpoint->setXY( mainpoint->x() + mainpoint->x() - pnt->x(),
		mainpoint->y() + mainpoint->y() - pnt->y() );
}

// Undo/Redo system
void ASSDrawCanvas::AddUndo( wxString desc )
{
	PrepareUndoRedo(_undo, false, _T(""), desc);
	history.Push(_undo);
	m_frame->UpdateUndoRedoMenu();
}

bool ASSDrawCanvas::UndoOrRedo(bool isundo)
{
	if (isundo ? !history.CanUndo() : !history.CanRedo())
		return false;
	const wxString description = isundo ? history.TopUndo().desc : history.TopRedo().desc;
	UndoRedo current;
	PrepareUndoRedo(current, true, GenerateASS(), description);
	UndoRedo restored;
	const bool changed = isundo ? history.Undo(current, restored) : history.Redo(current, restored);
	if (!changed)
		return false;
	restored.Export(this);

	// reset some values before refreshing
	mousedownAt_point = NULL;
	pointedAt_point = NULL;
	SetHighlighted ( NULL, NULL );
	ClearPointsSelection();
	RefreshDisplay();
	RefreshUndocmds();
	return true;
}

bool ASSDrawCanvas::Undo()
{
	return UndoOrRedo( true );
}

bool ASSDrawCanvas::Redo()
{
	return UndoOrRedo( false );
}

wxString ASSDrawCanvas::GetTopUndo()
{
	if (!history.CanUndo())
		return _T("");
	else
		return wxGetTranslation(history.TopUndo().desc);
}

wxString ASSDrawCanvas::GetTopRedo()
{
	if (!history.CanRedo())
		return _T("");
	else
		return wxGetTranslation(history.TopRedo().desc);
}

void ASSDrawCanvas::RefreshUndocmds()
{
	_undo.Import(this, true, GenerateASS());
}

void ASSDrawCanvas::PrepareUndoRedo(UndoRedo& ur, bool prestage, wxString cmds, wxString desc)
{
	ur.Import(this, prestage, cmds);
	ur.desc = desc;
}

// set command and point to highlight
void ASSDrawCanvas::SetHighlighted ( DrawCmd* cmd, Point* point )
{
     hilite_cmd = cmd;
     hilite_point = point;
}

int ASSDrawCanvas::SelectPointsWithin( int lx, int rx, int ty, int by, SELECTMODE smode )
{

	DrawCmdList::iterator iterate = cmds.begin();
	for (; iterate != cmds.end(); iterate++)
	{
		wxPoint wx = (*iterate)->m_point->ToWxPoint();

		if (wx.x >= lx && wx.x <= rx && wx.y >= ty && wx.y <= by)
			(*iterate)->m_point->isselected = (smode != DEL);
		else
	    	(*iterate)->m_point->isselected &= (smode != NEW);

		if ((*iterate)->m_point->isselected)
	    	selected_points.insert((*iterate)->m_point);
	    else
			selected_points.erase((*iterate)->m_point);

		PointList::iterator pnt_iterator = (*iterate)->controlpoints.begin();
		PointList::iterator end = (*iterate)->controlpoints.end();
		for (; pnt_iterator != end; pnt_iterator++)
		{
			wxPoint wx = (*pnt_iterator)->ToWxPoint();

			if (wx.x >= lx && wx.x <= rx && wx.y >= ty && wx.y <= by)
	    		(*pnt_iterator)->isselected = (smode != DEL);
		    else
	    		(*pnt_iterator)->isselected &= (smode != NEW);

			if ((*pnt_iterator)->isselected)
				selected_points.insert(*pnt_iterator);
			else
				selected_points.erase(*pnt_iterator);
		}
	}

	return selected_points.size();

}

void ASSDrawCanvas::ClearPointsSelection()
{
	if (!selected_points.empty())
	{
		PointSet::iterator piter = selected_points.begin();
		for (; piter != selected_points.end(); piter++)
		{
			(*piter)->isselected = false;
		}
		selected_points.clear();
	}
}

SELECTMODE ASSDrawCanvas::GetSelectMode(wxMouseEvent& event)
{
	SELECTMODE smode = NEW;
	bool CtrlDown = event.CmdDown();
	bool AltDown = event.AltDown();
	if (CtrlDown && !AltDown) smode = ADD;
	else if (!CtrlDown && AltDown) smode = DEL;
	return smode;
}

void ASSDrawCanvas::DoDraw( RendererBase& rbase, RendererPrimitives& rprim, RendererSolid& rsolid, agg::trans_affine& mtx )
{
	ASSDrawEngine::Draw_Clear( rbase );
	int ww, hh; GetClientSize(&ww, &hh);
	const double dpi_scale = GetDPIScaleFactor() >= 1.0 ? GetDPIScaleFactor() : 1.0;
	const auto scale_ui = [dpi_scale](double value) { return value * dpi_scale; };
	const auto dip = [this](int value) { return FromDIP(value); };

	if (HasBackgroundImage())
	{
		rasterizer.reset();
	    interpolator_type interpolator(bgimg.img_mtx);
	    BackgroundPixelFormat ipixfmt(bgimg.ibuf);
		span_gen_type spangen(ipixfmt, agg::rgba_pre(0, 0, 0, 0), interpolator);
		agg::conv_transform< agg::path_storage > bg_border(bgimg.bg_path, bgimg.path_mtx);
		agg::conv_clip_polygon< agg::conv_transform< agg::path_storage > > bg_clip(bg_border);
		bg_clip.clip_box(0, 0, ww, hh);
		rasterizer.add_path(bg_clip);
		agg::render_scanlines_aa(rasterizer, scanline, rbase, bgimg.spanalloc, spangen);
	}

	const std::vector<StyledSubshape> subshapes = BuildStyledSubshapes();
	for (std::vector<StyledSubshape>::const_iterator shape = subshapes.begin(); shape != subshapes.end(); ++shape)
	{
		agg::path_storage path;
		for (std::vector<DrawCmd*>::const_iterator command = shape->commands.begin(); command != shape->commands.end(); ++command)
			AddDrawCmdToAGGPathStorage(*command, path);
		agg::conv_transform< agg::path_storage > transformed(path, mtx);
		agg::conv_curve< agg::conv_transform< agg::path_storage > > curve(transformed);

		if (shape->style.fill_enabled)
		{
			rasterizer.reset();
			agg::conv_contour< agg::conv_curve< agg::conv_transform< agg::path_storage > > > fill(curve);
			rasterizer.add_path(fill);
			render_scanlines_aa_solid(rbase, ToAggColor(shape->style.fill_color, shape->style.fill_opacity));
		}

		if (shape->style.outline_enabled && shape->style.outline_width > 0.0)
		{
			rasterizer.reset();
			agg::conv_stroke< agg::conv_curve< agg::conv_transform< agg::path_storage > > > stroke(curve);
			stroke.width(shape->style.outline_width * pointsys->scale);
			rasterizer.add_path(stroke);
			render_scanlines_aa_solid(rbase, ToAggColor(shape->style.outline_color, shape->style.outline_opacity));
		}

		if (draw_mode == MODE_COLOR && coloring_target_selected && shape->start_command == coloring_target_shape)
		{
			rasterizer.reset();
			agg::conv_stroke< agg::conv_curve< agg::conv_transform< agg::path_storage > > > selection_stroke(curve);
			selection_stroke.width(scale_ui(2.0));
			rasterizer.add_path(selection_stroke);
			render_scanlines_aa_solid(rbase, rgba_selectpoint);
		}
	}

	if (!preview_mode)
	{
		// [0, 0]
		rasterizer.reset();
		agg::path_storage org_path;
		const double origin_cross_size = scale_ui(static_cast<double>(m_frame->sizes.origincross));
		org_path.move_to(0, origin_cross_size);
		org_path.line_to(0, -origin_cross_size);
		org_path.move_to(origin_cross_size, 0);
		org_path.line_to(-origin_cross_size, 0);
		agg::conv_transform< agg::path_storage > org_path_t(org_path, mtx);
	    agg::conv_curve< agg::conv_transform< agg::path_storage > > crosshair(org_path_t);
		agg::conv_stroke< agg::conv_curve< agg::conv_transform< agg::path_storage > > > chstroke(crosshair);
		rasterizer.add_path(chstroke);
		rsolid.color(rgba_origin);
		render_scanlines(rsolid, false);

		if (IsTransformMode() && isshapetransformable)
		{
			if (draw_mode == MODE_SCALEROTATE)
			{
				// rotation centerpoint
				rasterizer.reset();
				double len = scale_ui(10.0);
				org_path.free_all();
				org_path.move_to(rectcenter.x - len, rectcenter.y - len);
				org_path.line_to(rectcenter.x + len, rectcenter.y + len);
				org_path.move_to(rectcenter.x + len, rectcenter.y - len);
				org_path.line_to(rectcenter.x - len, rectcenter.y + len);
				agg::conv_stroke< agg::path_storage > cstroke(org_path);
				rasterizer.add_path(cstroke);
				agg::ellipse circ(rectcenter.x, rectcenter.y, len, len);
				agg::conv_stroke< agg::ellipse > c(circ);
				rasterizer.add_path(c);
				rsolid.color(rgba_origin);
				render_scanlines(rsolid, false);
			}

			rasterizer.reset();
			agg::path_storage org_path;
			org_path.move_to(rectbound2[0].x, rectbound2[0].y);
			org_path.line_to(rectbound2[1].x, rectbound2[1].y);
			org_path.line_to(rectbound2[2].x, rectbound2[2].y);
			org_path.line_to(rectbound2[3].x, rectbound2[3].y);
			org_path.line_to(rectbound2[0].x, rectbound2[0].y);
			agg::conv_stroke<agg::path_storage> chstroke(org_path);
			chstroke.width(scale_ui(1.0));
			rsolid.color(rgba_origin);
			rasterizer.add_path(chstroke);
			if (rectbound2upd != -1)
			{
				agg::ellipse circ(rectbound2[rectbound2upd].x, rectbound2[rectbound2upd].y,
					kTransformHandleRadius, kTransformHandleRadius);
				agg::conv_contour< agg::ellipse > c(circ);
				rasterizer.add_path(c);
			}
			if (rectbound2upd2 != -1)
			{
				agg::ellipse circ(rectbound2[rectbound2upd2].x, rectbound2[rectbound2upd2].y,
					kTransformHandleRadius, kTransformHandleRadius);
				agg::conv_contour< agg::ellipse > c(circ);
				rasterizer.add_path(c);
			}
			render_scanlines(rsolid, false);
		}
		else
		{
			// outlines
			agg::conv_stroke< agg::conv_transform< agg::path_storage > > bguidestroke(*rb_path);
			bguidestroke.width(scale_ui(1.0));
			rsolid.color(rgba_guideline);
			rasterizer.add_path(bguidestroke);
			render_scanlines(rsolid);

			agg::conv_stroke< agg::conv_curve< agg::conv_transform< agg::path_storage > > > stroke(*rm_curve);
			stroke.width(scale_ui(1.0));
			rsolid.color(rgba_outline);
			rasterizer.add_path(stroke);
			render_scanlines(rsolid);

			double diameter = scale_ui(EditorPointDiameter(pointsys->scale));
			double radius = diameter / 2.0;
			// hilite
			if (hilite_cmd && hilite_cmd->type != M)
			{
				rasterizer.reset();
				agg::path_storage h_path;
				AddDrawCmdToAGGPathStorage(hilite_cmd, h_path, HILITE);
				agg::conv_transform< agg::path_storage> h_path_trans(h_path, mtx);
			    agg::conv_curve< agg::conv_transform< agg::path_storage> > curve(h_path_trans);
				agg::conv_dash< agg::conv_curve< agg::conv_transform< agg::path_storage > > > d(curve);
				d.add_dash(scale_ui(10.0), scale_ui(5.0));
				agg::conv_stroke< agg::conv_dash< agg::conv_curve< agg::conv_transform< agg::path_storage > > > > stroke(d);
				stroke.width(scale_ui(3.0));
				rsolid.color(rgba_outline);
				rasterizer.add_path(stroke);
				render_scanlines(rsolid);
			}

			// m_point
			rasterizer.reset();
			DrawCmdList::iterator ci = cmds.begin();
			while (ci != cmds.end())
			{
				double lx = (*ci)->m_point->x() * pointsys->scale + pointsys->originx - radius;
				double ty = (*ci)->m_point->y() * pointsys->scale + pointsys->originy - radius;
				agg::path_storage sqp = agghelper::RectanglePath(lx, lx + diameter, ty, ty + diameter);
				agg::conv_contour< agg::path_storage > c(sqp);
				rasterizer.add_path(c);
				ci++;
			}
			render_scanlines_aa_solid(rbase, rgba_mainpoint);

			// control_points
			rasterizer.reset();
			ci = cmds.begin();
			while (ci != cmds.end())
			{
				PointList::iterator pi = (*ci)->controlpoints.begin();
				while (pi != (*ci)->controlpoints.end())
				{
					agg::ellipse circ((*pi)->x() * pointsys->scale + pointsys->originx,
						(*pi)->y() * pointsys->scale + pointsys->originy, radius, radius);
					agg::conv_contour< agg::ellipse > c(circ);
					rasterizer.add_path(c);
					pi++;
				}
				ci++;
			}
			render_scanlines_aa_solid(rbase, rgba_controlpoint);

			// selection
			rasterizer.reset();
			PointSet::iterator si = selected_points.begin();
			while (si != selected_points.end())
			{
				agg::ellipse circ((*si)->x() * pointsys->scale + pointsys->originx,
					(*si)->y() * pointsys->scale + pointsys->originy, radius + scale_ui(3.0), radius + scale_ui(3.0));
				agg::conv_stroke< agg::ellipse > s(circ);
				rasterizer.add_path(s);
				si++;
			}
			render_scanlines_aa_solid(rbase, rgba_selectpoint);

			// hover
			if (hilite_point)
			{
				rasterizer.reset();
				agg::ellipse circ(hilite_point->x() * pointsys->scale + pointsys->originx,
					hilite_point->y() * pointsys->scale + pointsys->originy, radius + scale_ui(3.0), radius + scale_ui(3.0));
				agg::conv_stroke< agg::ellipse > s(circ);
				s.width(scale_ui(2.0));
				rasterizer.add_path(s);
				render_scanlines_aa_solid(rbase, rgba_selectpoint);

				rasterizer.reset();
				agg::gsv_text t;
				t.flip(true);
				t.size(scale_ui(8.0));
				wxPoint pxy = hilite_point->ToWxPoint(true);
				t.start_point(pxy.x + dip(5), pxy.y - dip(5));
				t.text(wxString::Format(_T("%d,%d"), hilite_point->x(), hilite_point->y()).mb_str(wxConvUTF8));
				agg::conv_stroke< agg::gsv_text > pt(t);
				pt.line_cap(agg::round_cap);
				pt.line_join(agg::round_join);
				pt.width(scale_ui(1.5));
				rasterizer.add_path(pt);
				rsolid.color(agg::rgba(0,0,0));
				render_scanlines(rsolid, false);

				rasterizer.reset();
				agg::path_storage sb_path;
				sb_path.move_to(pxy.x, 0);
				sb_path.line_to(pxy.x, hh);
				sb_path.move_to(0, pxy.y);
				sb_path.line_to(ww, pxy.y);
			    agg::conv_curve< agg::path_storage > curve(sb_path);
				agg::conv_dash< agg::conv_curve< agg::path_storage > > d(curve);
				d.add_dash(scale_ui(10.0), scale_ui(5.0));
				agg::conv_stroke< agg::conv_dash< agg::conv_curve< agg::path_storage > > > stroke(d);
				stroke.width(scale_ui(1.0));
				rsolid.color(agg::rgba(0,0,0,0.5));
				rasterizer.add_path(stroke);
				render_scanlines(rsolid);
			}

			// selection box
			if (lastDrag_left)
			{
				double x1 = lastDrag_left->x, y1 = lastDrag_left->y;
				double x2 = dragAnchor_left->x, y2 = dragAnchor_left->y;
				double lx, rx, ty, by;
				if (x1 < x2) lx = x1, rx = x2;
				else lx = x2, rx = x1;
				if (y1 < y2) ty = y1, by = y2;
				else ty = y2, by = y1;
				rasterizer.reset();
				agg::path_storage sb_path = agghelper::RectanglePath(lx, rx, ty, by);
			    agg::conv_curve< agg::path_storage > curve(sb_path);
				agg::conv_dash< agg::conv_curve< agg::path_storage > > d(curve);
				d.add_dash(scale_ui(10.0), scale_ui(5.0));
				agg::conv_stroke< agg::conv_dash< agg::conv_curve< agg::path_storage > > > stroke(d);
				stroke.width(scale_ui(1.0));
				rsolid.color(agg::rgba(0,0,0,0.5));
				rasterizer.add_path(stroke);
				render_scanlines(rsolid);
			}
		}
	}

	// ruler
	int w, h;
	GetClientSize( &w, &h );
	double scale = pointsys->scale;
	double coeff = 9 / scale + 1;
	int numdist = (int) floor(coeff) * 5;
	{
		rsolid.color(rgba_ruler_h);
		rasterizer.reset();
		agg::path_storage rlr_path;
		double start = pointsys->originx;
		int t = - (int) floor(start / scale);
		double s = (start + t * scale);
		double collect = s;
		int len;

		for (; s <= w; s += scale)
		{
			bool longtick = t % numdist == 0;
			if (longtick)
			{
				len = dip(10);
				agg::gsv_text txt;
				txt.flip(true);
				txt.size(scale_ui(6.0));
				txt.start_point(s, dip(20));
				txt.text(wxString::Format(_T("%d"), t).mb_str(wxConvUTF8));
				agg::conv_stroke< agg::gsv_text > pt(txt);
				pt.width(scale_ui(1.0));
				rasterizer.add_path(pt);
			}
			else
				len = dip(5);
			t++ ;
			collect += scale;
			if (collect > scale_ui(5.0))
			{
				collect = 0.0;
				rlr_path.move_to(s, 0);
				rlr_path.line_to(s, len);
			}
		}
		agg::conv_stroke< agg::path_storage > rlr_stroke(rlr_path);
		rlr_stroke.width(scale_ui(1.0));
		rasterizer.add_path(rlr_stroke);
		render_scanlines(rsolid, false);
	}
	{
		rasterizer.reset();
		rsolid.color(rgba_ruler_v);
		agg::path_storage rlr_path;
		double start = pointsys->originy;
		int t = - (int) floor(start / scale);
		double s = (start + t * scale);
		double collect = 0;
		int len;

		for (; s <= h; s += scale)
		{
			bool longtick = t % numdist == 0;
			if (longtick)
			{
				len = dip(10);
				agg::gsv_text txt;
				txt.flip(true);
				txt.size(scale_ui(6.0));
				txt.start_point(dip(12), s);
				txt.text(wxString::Format(_T("%d"), t).mb_str(wxConvUTF8));
				agg::conv_stroke< agg::gsv_text > pt(txt);
				pt.width(scale_ui(1.0));
				rasterizer.add_path(pt);
			}
			else
				len = dip(5);
			t++ ;
			collect += scale;
			if (collect > scale_ui(5.0))
			{
				collect = 0.0;
				rlr_path.move_to(0, s);
				rlr_path.line_to(len, s);
			}
		}
		agg::conv_stroke< agg::path_storage > rlr_stroke(rlr_path);
		rlr_stroke.width(scale_ui(1.0));
		rasterizer.add_path(rlr_stroke);
		render_scanlines(rsolid, false);
	}

}

void ASSDrawCanvas::ReceiveBackgroundImageFileDropEvent(const wxString& filename)
{
	const wxString shortfname = wxFileName::FileName(filename).GetFullName();
	m_frame->SetStatusText(wxString::Format(_("Loading '%s' as canvas background ..."), shortfname), 1);
	wxImage img;
	img.LoadFile(filename);
	if (img.IsOk())
	{
		SetBackgroundImage(img, filename);
		m_frame->SetStatusText(_("Canvas background loaded"), 1);
	}
	else m_frame->SetStatusText(wxString(), 1);
}

void ASSDrawCanvas::RemoveBackgroundImage()
{
	if (bgimg.bgimg) delete bgimg.bgimg;
	bgimg.bgimg = NULL;
	bgimg.pixels.clear();
	bgimg.ibuf.attach(nullptr, 0, 0, 0);
	bgimg.bgimgfile = _T("");
	RefreshDisplay();
	drag_mode = DRAGMODE();
	bgimg.alpha_dlg->Show(false);
	m_frame->UpdateFrameUI();
}

void ASSDrawCanvas::SetBackgroundImage(const wxImage& img, wxString fname, bool ask4alpha)
{
	if (!img.IsOk()) return;
	// Copy before deleting the previous image, including for a caller passing it
	// back to us. Opacity/theme changes must never modify these source pixels.
	wxImage* replacement = new wxImage(img.Copy());
	if (bgimg.bgimg) delete bgimg.bgimg;
	bgimg.bgimg = replacement;
	bgimg.bgimgfile = fname;
	const long opacity = std::clamp(m_frame->alphas.dfltimgopac, 0L, 255L);
	PrepareBackgroundBitmap(1.0 - opacity / 255.0);
    UpdateBackgroundImgScalePosition(true);
	RefreshDisplay();
	m_frame->UpdateFrameUI();
	if (ask4alpha && m_frame->behaviors.autoaskimgopac)
		AskUserForBackgroundAlpha();
}

void ASSDrawCanvas::AskUserForBackgroundAlpha()
{
	bgimg.alpha_slider->SetValue((int) (100 - bgimg.alpha * 100));
	bgimg.alpha_slider->SetMinSize(wxSize(FromDIP(280), -1));
	bgimg.alpha_dlg->Fit();
	AssdrawTheme::ApplyControls(bgimg.alpha_dlg);
	bgimg.alpha_dlg->Show();
	SetFocus();
}

void ASSDrawCanvas::PrepareBackgroundBitmap(double alpha)
{
	if (alpha >= 0.0 && alpha <= 1.0) bgimg.alpha = alpha;
	if (!bgimg.bgimg || !bgimg.bgimg->IsOk()) return;
	const wxImage& image = *bgimg.bgimg;
	const unsigned w = image.GetWidth(), h = image.GetHeight();
	const size_t count = size_t(w) * h;
	bgimg.pixels.resize(count * 3);
	const unsigned char* source = image.GetData();
	const unsigned char* source_alpha = image.HasAlpha() ? image.GetAlpha() : nullptr;
	const bool masked = image.HasMask();
	const int opacity = int(std::lround(255.0 * (1.0 - bgimg.alpha)));
	const int background[] = {color_bg.r, color_bg.g, color_bg.b};
	for (size_t i = 0; i < count; ++i) {
		int a = source_alpha ? source_alpha[i] : 255;
		if (masked && source[i * 3] == image.GetMaskRed()
			&& source[i * 3 + 1] == image.GetMaskGreen()
			&& source[i * 3 + 2] == image.GetMaskBlue()) a = 0;
		a = (a * opacity + 127) / 255;
		for (size_t channel = 0; channel < 3; ++channel)
			bgimg.pixels[i * 3 + channel] =
				(source[i * 3 + channel] * a + background[channel] * (255 - a) + 127) / 255;
	}
	// AGG retains this pointer between paints. Never attach it to wxPixelData:
	// unlocking a native bitmap can release or relocate its temporary buffer.
	bgimg.ibuf.attach(bgimg.pixels.data(), w, h, int(w) * 3);
	bgimg.bg_path = agghelper::RectanglePath(0, w, 0, h);
}

void ASSDrawCanvas::UpdateBackgroundImgScalePosition(bool firsttime)
{
	if (!HasBackgroundImage()) return;
	// transform the enclosing polygon
	unsigned w = bgimg.bgimg->GetWidth(), h = bgimg.bgimg->GetHeight();
	bgimg.bg_path = agghelper::RectanglePath(0, w, 0, h);
    // linear interpolation on image buffer
    wxRealPoint center, disp;
    double scale;
    if (firsttime) // first time
    {
	    bgimg.img_mtx = agg::trans_affine();
	    scale = 1.0;
		center = wxRealPoint(0.0, 0.0);
	    disp = wxRealPoint(0.0, 0.0);
	    bgimg.path_mtx = bgimg.img_mtx;
	}
	else
	{
		wxRealPoint d_disp(bgimg.new_disp.x - bgimg.disp.x, bgimg.new_disp.y - bgimg.disp.y);
		scale = bgimg.new_scale;
		disp = bgimg.disp;
		center = bgimg.new_center;
		if (bgimg.scale == scale)
		{
		    bgimg.img_mtx.invert();
		    bgimg.img_mtx *= agg::trans_affine_translation(d_disp.x, d_disp.y);
			d_disp.x /= scale;
			d_disp.y /= scale;
		}
		else
		{
			d_disp.x /= scale;
			d_disp.y /= scale;
		    bgimg.img_mtx = agg::trans_affine();
			disp.x += (center.x - bgimg.center.x) * (1.0 - 1.0 / bgimg.scale);
			disp.y += (center.y - bgimg.center.y) * (1.0 - 1.0 / bgimg.scale);
		    bgimg.img_mtx *= agg::trans_affine_translation(-center.x + disp.x, -center.y + disp.y);
		    bgimg.img_mtx *= agg::trans_affine_scaling(scale);
		    bgimg.img_mtx *= agg::trans_affine_translation(center.x + d_disp.x, center.y + d_disp.y);
		}
	    bgimg.path_mtx = bgimg.img_mtx;
	    bgimg.img_mtx.invert();
	    disp.x += d_disp.x;
	    disp.y += d_disp.y;
	}
    //update
    bgimg.scale = scale;
    bgimg.center = center;
    bgimg.disp = disp;
    bgimg.new_scale = scale;
    bgimg.new_center = center;
    bgimg.new_disp = disp;
}

bool ASSDrawCanvas::GetBackgroundInfo(unsigned& w, unsigned& h, wxRealPoint& disp, double& scale)
{
	if (!HasBackgroundImage()) return false;
	w = bgimg.bgimg->GetWidth(), h = bgimg.bgimg->GetHeight();
	double t, l;
	agg::conv_transform<agg::path_storage, agg::trans_affine> trr(bgimg.bg_path, bgimg.path_mtx);
	trr.rewind(0);
	trr.vertex(&l, &t);
	disp = wxRealPoint(l, t);
	scale = bgimg.scale;
	return true;
}

void ASSDrawCanvas::UpdateNonUniformTransformation()
{
	double bound[8] = {
		rectbound2[0].x, rectbound2[0].y,
		rectbound2[1].x, rectbound2[1].y,
		rectbound2[2].x, rectbound2[2].y,
		rectbound2[3].x, rectbound2[3].y };
	agg::path_storage trans;
	unsigned vertices = backupcmds.total_vertices();

	agg::trans_bilinear trans_b(rectbound[0].x, rectbound[0].y, rectbound[2].x, rectbound[2].y, bound);
	agg::conv_transform<agg::path_storage, agg::trans_bilinear> transb(backupcmds, trans_b);
	transb.rewind(0);
	for (int i = 0; i < vertices; i++)
	{
		double x, y;
		transb.vertex(&x, &y);
		trans.move_to(x, y);
	}

	trans.rewind(0);
	for (DrawCmdList::iterator iterate = cmds.begin(); iterate != cmds.end(); iterate++)
	{
		DrawCmd* cmd = (*iterate);
		for (PointList::iterator iterate2 = cmd->controlpoints.begin(); iterate2 != cmd->controlpoints.end(); iterate2++)
		{
	        double x, y;
	        trans.vertex(&x, &y);
	        int wx, wy;
			pointsys->FromWxPoint ( wxPoint((int)x, (int)y), wx, wy );
			(*iterate2)->setXY(wx, wy);
		}
	    double x, y;
	    trans.vertex(&x, &y);
	    int wx, wy;
		pointsys->FromWxPoint ( wxPoint((int)x, (int)y), wx, wy );
		(*iterate)->m_point->setXY(wx, wy);
	}

}

void ASSDrawCanvas::CustomOnKeyDown(wxKeyEvent &event)
{
	int keycode = event.GetKeyCode();
	double scrollamount = (event.GetModifiers() == wxMOD_CMD? 10.0:1.0);
    if (event.GetModifiers() == wxMOD_SHIFT)
	{
		MODE d_mode = GetDrawMode();
        if ((int) d_mode > (int) MODE_ARR && (int) d_mode < (int) MODE_SCALEROTATE)
		{
			mode_b4_shift = d_mode;
			SetDrawMode( MODE_ARR );
			m_frame->UpdateFrameUI();
		}
    }
    else
    {
		switch (keycode)
		{
		case WXK_PAGEUP:
			ChangeZoomLevel( 1.0 /scrollamount, wxPoint( (int) pointsys->originx, (int) pointsys->originy ) );
			RefreshDisplay();
			break;
		case WXK_PAGEDOWN:
			ChangeZoomLevel( - 1.0 /scrollamount, wxPoint( (int) pointsys->originx, (int) pointsys->originy ) );
			RefreshDisplay();
			break;
		case WXK_UP:
			MoveCanvas(0.0, -scrollamount);
			RefreshDisplay();
			break;
		case WXK_DOWN:
			MoveCanvas(0.0, scrollamount);
			RefreshDisplay();
			break;
		case WXK_LEFT:
			MoveCanvas(-scrollamount, 0.0);
			RefreshDisplay();
			break;
		case WXK_RIGHT:
			MoveCanvas(scrollamount, 0.0);
			RefreshDisplay();
			break;
		case WXK_TAB:
			if (mousedownAt_point == NULL && !IsTransformMode() && cmds.size() > 0)
			{
				if (pointedAt_point == NULL)
				{
					Point *nearest = NULL;
					double dist = 0.0;
					DrawCmdList::iterator it = cmds.begin();
					while(it != cmds.end())
					{
						wxPoint point = (*it)->m_point->ToWxPoint();
						double distance = sqrt(pow(double(point.x - mouse_point.x), 2) + pow(double(point.y - mouse_point.y), 2));
						if (nearest == NULL || distance < dist)
						{
							nearest = (*it)->m_point;
							dist = distance;
						}
						PointList::iterator it2 = (*it)->controlpoints.begin();
						while (it2 != (*it)->controlpoints.end())
						{
							wxPoint point = (*it2)->ToWxPoint();
							double distance = sqrt(pow((double)point.x - mouse_point.x, 2) + pow((double)point.y - mouse_point.y, 2));
							if (nearest == NULL || distance < dist)
							{
								nearest = (*it2);
								dist = distance;
							}
							it2++;
						}
						it++;
					}
					if (nearest != NULL)
					{
						wxPoint point = nearest->ToWxPoint();
						WarpPointer(point.x, point.y);
					}
				}
				else
				{
					Point *warpto = NULL;
					if (pointedAt_point->type == MP && pointedAt_point->cmd_next)
					{
						if (pointedAt_point->cmd_next->controlpoints.size() > 0)
							warpto = pointedAt_point->cmd_next->controlpoints.front();
						else
							warpto = pointedAt_point->cmd_next->m_point;
					}
					else
					{
						PointList::iterator it = pointedAt_point->cmd_main->controlpoints.begin();
						while (*it != pointedAt_point) it++;
						it++;
						if (it == pointedAt_point->cmd_main->controlpoints.end())
							warpto = pointedAt_point->cmd_main->m_point;
						else
							warpto = *it;
					}
					if (warpto == NULL)
						warpto = cmds.front()->m_point;
					wxPoint point = warpto->ToWxPoint();
					WarpPointer(point.x, point.y);
				}
			}
			break;
		default:
	        event.Skip();
		}
	}
}

void ASSDrawCanvas::CustomOnKeyUp(wxKeyEvent &event)
{
	if (event.GetModifiers() != wxMOD_SHIFT && (int) mode_b4_shift > (int) MODE_ARR)
	{
		SetDrawMode( mode_b4_shift );
		m_frame->UpdateFrameUI();
		mode_b4_shift = MODE_ARR;
	}
}

void ASSDrawCanvas::OnAlphaSliderChanged(wxScrollEvent &event)
{
	double pos = (double) event.GetPosition();
	PrepareBackgroundBitmap(1.0 - pos / 100.0);
	RefreshDisplay();
}

void ASSDrawCanvas::CustomOnMouseCaptureLost(wxMouseCaptureLostEvent &)
{
	if (capturemouse_left)
		ProcessOnMouseLeftUp();

	if (capturemouse_right)
		ProcessOnMouseRightUp();
}

void UndoRedo::Import(ASSDrawCanvas *canvas, bool prestage, wxString cmds)
{
	if (prestage)
	{
		this->cmds = cmds;
		this->c1cont = canvas->PrepareC1ContData();
	}
}

void UndoRedo::Export(ASSDrawCanvas *canvas)
{
	canvas->ParseASS(this->cmds, false);
	DrawCmdList::iterator it1 = canvas->cmds.begin();
	std::vector< bool >::iterator it2 = this->c1cont.begin();
	for(; it1 != canvas->cmds.end() && it2 != this->c1cont.end(); it1++, it2++)
		if (*it2 && (*it1)->type == B)
			static_cast<DrawCmd_B*>(*it1)->C1Cont = true;

	// Reference images and canvas view state are deliberately not restored.
	if (canvas->IsTransformMode())
	{
		canvas->SetDrawMode(canvas->draw_mode);
	}
}
