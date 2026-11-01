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
// Name:        canvas.hpp
// Purpose:     header file for ASSDraw main canvas class
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////

#pragma once

#include <map>
#include <vector>

#include "engine.hpp"
#include "enums.hpp"
#include "drawing_history.hpp"
#include "shape_style.hpp"

#include <wx/dnd.h>
#include <wx/splitter.h>
#include <wx/clntdata.h>

#include "agg_span_allocator.h"
#include "agg_span_interpolator_linear.h"
#include "agg_span_image_filter_rgb.h"
#include "agg_span_image_filter_rgba.h"
#include "agg_image_accessors.h"
#include "agg_conv_clip_polygon.h"

class ASSDrawFrame;
class ASSDrawCanvas;

struct UndoRedo
{
	wxString cmds;
	wxString desc;
	std::vector< bool > c1cont;

	void Import(ASSDrawCanvas *canvas, bool prestage, wxString cmds = _T(""));
	void Export(ASSDrawCanvas *canvas);
	
};

// A visible ASS drawing component. `commands` contains the outer contour and
// directly nested, oppositely-wound hole contours, which must share an ASS
// drawing run to preserve their fill relationship.
struct StyledSubshape
{
	DrawCmd* start_command = NULL;
	std::vector<DrawCmd*> commands;
	std::vector< std::vector<wxRealPoint> > contours;
	ShapeStyle style;
};

// for multiple point selection
enum SELECTMODE { NEW, ADD, DEL };

class ASSDrawCanvas: public ASSDrawEngine, public wxClientData
{
public:
	ASSDrawCanvas( wxWindow *parent, ASSDrawFrame *frame, int extraflags = 0 );

    // destructor
    ~ASSDrawCanvas();

    virtual void ResetEngine(bool addM);
	virtual void SetPreviewMode( bool mode );
	virtual bool IsPreviewMode() { return preview_mode; }
	virtual void ParseASS(wxString str, bool addundo = false);
	virtual wxString GenerateASS() override;
	virtual wxString GenerateDrawingASS();
	using ASSDrawEngine::AppendCmd;
	using ASSDrawEngine::InsertCmd;
	virtual DrawCmd* AppendCmd(DrawCmd* cmd) override;
	virtual void InsertCmd(DrawCmd* cmd, DrawCmd* after) override;
	virtual bool DeleteCommand(DrawCmd* cmd) override;
	virtual void SetShapeStyle(const ShapeStyle& new_style);
	virtual const ShapeStyle& GetShapeStyle() const { return shape_style; }
	virtual void ApplyShapeStyle();
	void UpdateTranslations();

	virtual void SetDrawMode( MODE mode );
	virtual MODE GetDrawMode() { return draw_mode; }
	virtual bool IsTransformMode();
	virtual void SetDragMode( DRAGMODE mode );
	virtual DRAGMODE GetDragMode() { return drag_mode; }
	virtual void RefreshDisplay();
	virtual bool CanZoom();
	virtual bool CanMove();
	
    virtual void OnMouseMove(wxMouseEvent &event);
    virtual void OnMouseLeftUp(wxMouseEvent &event);
    virtual void OnMouseLeftDown(wxMouseEvent &event);
	virtual void OnMouseLeftDClick(wxMouseEvent &event);
    virtual void OnMouseRightUp(wxMouseEvent &event);
    virtual void OnMouseRightDown(wxMouseEvent &event);
    virtual void OnMouseRightDClick(wxMouseEvent &event);
    virtual void OnMouseWheel(wxMouseEvent &event);
    virtual void OnDPIChanged(wxDPIChangedEvent &event);
    virtual void CustomOnKeyDown(wxKeyEvent &event);
    virtual void CustomOnKeyUp(wxKeyEvent &event);
	virtual void ChangeZoomLevel(double zoomamount, wxPoint bgzoomctr);
	virtual void ChangeZoomLevelTo(double zoom, wxPoint bgzoomctr);
	virtual void ChangeDrawingZoomLevel(double zoom);
	virtual void ChangeBackgroundZoomLevel(double zoom, wxRealPoint newcenter);
	virtual void MoveCanvas(double xamount, double yamount);
	virtual void MoveCanvasOriginTo(double originx, double originy);
	virtual void MoveCanvasDrawing(double xamount, double yamount);
	virtual void MoveCanvasBackground(double xamount, double yamount);
	virtual void OnSelect_ConvertLineToBezier(wxCommandEvent& WXUNUSED(event));
	virtual void OnSelect_ConvertBezierToLine(wxCommandEvent& WXUNUSED(event));
	virtual void OnSelect_C1ContinuityBezier(wxCommandEvent& WXUNUSED(event));
	virtual void OnSelect_Move00Here(wxCommandEvent& WXUNUSED(event));
	void OnAlphaSliderChanged(wxScrollEvent &event);
	
	// to replace _PointSystem() that has been made protected
	double GetScale() { return pointsys->scale; }
	double GetOriginX() { return pointsys->originx; }
	double GetOriginY() { return pointsys->originy; }

	// undo/redo system
	virtual void AddUndo( wxString desc );
	virtual bool UndoOrRedo(bool isundo);
	virtual bool Undo();
	virtual bool Redo();
	virtual wxString GetTopUndo();
	virtual wxString GetTopRedo();
	virtual void RefreshUndocmds();

	virtual bool HasBackgroundImage() { return bgimg.bgimg != NULL; }
	virtual void RemoveBackgroundImage();
	virtual void ReceiveBackgroundImageFileDropEvent(const wxString& filename);
	virtual void SetBackgroundImage(const wxImage& img, wxString fname = _T("<clipboard>"), bool ask4alpha = true);
	virtual void PrepareBackgroundBitmap(double alpha);
	virtual void AskUserForBackgroundAlpha();
	virtual bool GetBackgroundInfo(unsigned& w, unsigned& h, wxRealPoint& disp, double& scale);
	
	agg::rgba rgba_shape_normal, rgba_outline, rgba_guideline;
	agg::rgba rgba_mainpoint, rgba_controlpoint, rgba_selectpoint;
	agg::rgba rgba_origin, rgba_ruler_h, rgba_ruler_v;
	
protected:

	typedef PixelFormat::AGGType::color_type color_type;
	typedef agg::span_interpolator_linear<> interpolator_type;
	typedef agg::span_image_filter_rgb_bilinear_clip<PixelFormat::AGGType, interpolator_type> span_gen_type;
	
    // The GUI window
	ASSDrawFrame* m_frame;

	// highlight mechanism
	DrawCmd* hilite_cmd;
	Point* hilite_point;
		
	// mouse capture
	bool capturemouse_left, capturemouse_right;
	virtual void CustomOnMouseCaptureLost(wxMouseCaptureLostEvent &event);
    virtual void ProcessOnMouseLeftUp();
    virtual void ProcessOnMouseRightUp();

	// selection mechanism
	PointSet selected_points;

	// if it has status bar
	bool hasStatusBar;

	// some mouse readings
    Point* mousedownAt_point;
	Point* pointedAt_point;
	Point* dblclicked_point_right;
	wxPoint mouse_point;

	// The wxPoint being dragged by left button
    wxPoint* dragAnchor_left;
	wxPoint* lastDrag_left;

	// The wxPoint being dragged by right button
    wxPoint* dragAnchor_right;
	wxPoint* lastDrag_right;
	
	// true if the drawing origin (0, 0) is being dragged
    bool dragOrigin;

	// The newest command being initialized thru dragging action
    DrawCmd* newcommand;

	// the draw mode
    MODE draw_mode;
    DRAGMODE drag_mode;

    // holding shift key temporarily switches to drag mode (MODE_ARR)
    // so we want to save the mode before the key-down to restore it on key-up
    MODE mode_b4_shift;

	// true if preview mode (i.e don't draw anything except the shape itself;
    // also draw the shape as closed)
    bool preview_mode;

	// background image!
	struct
	{
		agg::rendering_buffer ibuf;
		wxImage *bgimg;
		wxBitmap *bgbmp;
		wxString bgimgfile;
		agg::path_storage bg_path;		
		agg::span_allocator<color_type> spanalloc;
		//span_gen_type spangen;
        agg::trans_affine img_mtx, path_mtx;
        
        wxRealPoint disp, center, new_disp, new_center;
        double scale, new_scale, alpha;
		wxDialog* alpha_dlg;
		wxSlider* alpha_slider;
	} bgimg;
	
	// Undo/redo stores document geometry only. Reference images and view state
	// intentionally do not participate in drawing undo/redo.
	DrawingHistory<UndoRedo> history;
	UndoRedo _undo;
	ShapeStyle shape_style;
	std::map<DrawCmd*, ShapeStyle> subshape_styles;

	// last action and commands (for undo/redo system)
	wxString undodesc;
	
	wxString oldasscmds;

	// was preview_mode
	//bool was_preview_mode;

	PointSystem* _PointSystem() { return pointsys; }

	// for Undo/Redo system
	virtual void PrepareUndoRedo(UndoRedo& ur, bool prestage, wxString cmds, wxString desc);
	
	// -------------------- points highlight/selection ---------------------------

	// set command and point to highlight
	virtual void SetHighlighted ( DrawCmd* cmd, Point* point );

	// Find an editor point in device pixels.  This deliberately does not use
	// ASSDrawEngine::PointAt/ControlAt: their draw-coordinate comparison makes
	// the interactive target shrink to less than a pixel when zoomed out.
	virtual Point* FindPointAtScreenPosition(const wxPoint& position, bool control_point) const;
	virtual bool IsScreenPositionInFilledShape(const wxPoint& position) const;
	virtual DrawCmd* FindSubshapeAtScreenPosition(const wxPoint& position) const;
	virtual void SelectColoringTarget(const wxPoint& position);
	virtual void ShowColorSelector();
	virtual std::vector<StyledSubshape> BuildStyledSubshapes() const;
	virtual ShapeStyle GetSubshapeStyle(DrawCmd* start_command) const;
	virtual void SetSubshapeStyle(DrawCmd* start_command, const ShapeStyle& style);

	// selects all points within (lx, ty) , (rx, by) returns # of selected points
	virtual int SelectPointsWithin( int lx, int rx, int ty, int by, SELECTMODE smode = NEW );
	virtual void ClearPointsSelection();
	virtual SELECTMODE GetSelectMode(wxMouseEvent &event);
	
	// -------------------- misc ---------------------------

	// non-uniform transformation
	virtual bool InitiateDraggingIfTransformMode();
	virtual void UpdateTranformModeRectCenter();
	virtual bool GetThe4thPoint(double ox, double oy, double a1x, double a1y, double a2x, double a2y, double *x, double *y);
	enum { NONE, LEFT, RIGHT } backupowner;
	agg::path_storage backupcmds;
	int rectbound2upd, rectbound2upd2;
	wxRealPoint rectbound[4], rectbound2[4], backup[4], rectcenter;
	bool isshapetransformable;
	bool coloring_target_selected;
	DrawCmd* coloring_target_shape;

	// do the real drawing
	virtual void DoDraw( RendererBase& rbase, RendererPrimitives& rprim, RendererSolid& rsolid, agg::trans_affine& mtx );

	// update background image scale & position
	virtual void UpdateBackgroundImgScalePosition(bool firsttime = false);
	
	// perform extra stuff other than calling ASSDrawEngine::ConnectSubsequentCmds
	virtual void ConnectSubsequentCmds (DrawCmd* cmd1, DrawCmd* cmd2);

	// make sure the c1 continuity is followed after performing a drag-point action
	virtual void EnforceC1Continuity (DrawCmd* cmd, Point* pnt);
	
	// after the bounding quadrangle has changed, update the shape to fill up inside it
	virtual void UpdateNonUniformTransformation();

	friend struct UndoRedo;
	
	DECLARE_EVENT_TABLE()
};

class ASSDrawFileDropTarget : public wxFileDropTarget
{
public:
	ASSDrawFileDropTarget(ASSDrawCanvas *canvas): wxFileDropTarget()
	{
		m_canvas = canvas;
	}

	virtual bool OnDropFiles(wxCoord x, wxCoord y, const wxArrayString& filenames)
	{
		m_canvas->ReceiveBackgroundImageFileDropEvent(filenames.Item(0));
		return true;
	}

protected:
	ASSDrawCanvas *m_canvas;

};
