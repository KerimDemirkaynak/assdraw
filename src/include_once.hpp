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
// Name:        include_once.hpp
// Purpose:     A header file that is included exactly once for the whole code
//              (supposed to be included by assdraw.cpp only)
//              Put must-define-exactly-once codes here
// Author:      ai-chan
// Created:     08/26/06
// Copyright:   (c) ai-chan
// Licence:     3-clause BSD
///////////////////////////////////////////////////////////////////////////////

// tooltips
#define TIPS_CLEAR _("Clear the canvas and create a new drawing")
#define TIPS_EDITSRC _("Edit the source")
#define TIPS_PREVIEW _("Draw the shapes without enlarged points and control points")
#define TIPS_TRANSFORM _("Transform the drawing using matrix transformation")
#define TIPS_LIBRARY _("Shapes library")
#define TIPS_HELP _("Help! Help!")
#define TIPS_PASTE _("Depending on what's in the clipboard, import as drawing commands or background")
#define TIPS_UNDO _("Undo last action")
#define TIPS_REDO _("Redo last undo")
#define TIPS_ARR _("Drag mode")
#define TIPS_M _("Draw M mode (Close current shape and move the virtual pen to a new point)")
#define TIPS_N _("Draw N mode (Same as M but doesn't close the shape)")
#define TIPS_L _("Draw L mode (Straight line)")
#define TIPS_B _("Draw B mode (Cubic Bezier curve)")
#define TIPS_S _("Draw S mode (Spline)")
#define TIPS_P _("Draw P mode (Extends a spline with another point)")
#define TIPS_C _("Draw C mode (Close the last spline)")
#define TIPS_DEL _("Delete mode")
#define TIPS_NUTB _("Bilinear transformation mode: Drag the vertices to distort the shape; Dragging an edge moves two adjacent vertices together")
#define TIPS_SCALEROTATE _("Scale/Rotate mode: Drag a vertex or an edge to rescale the shape; Right-drag to rotate")
#define TIPS_DWG _("Right-dragging pans drawing, mousewheel zooms in/out drawing")
#define TIPS_BGIMG _("Right-dragging pans background, mousewheel zooms in/out background")
#define TIPS_BOTH _("Right-dragging pans drawing & background, mousewheel zooms in/out drawing & background")

#define TBNAME_DRAW _("Canvas")
#define TBNAME_MODE _("Drawing mode")
#define TBNAME_BGIMG _("Background")
#define TBNAME_ZOOM _("Zoom")

wxString ASSDrawTransformDlg::combo_templatesStrings[] = {
	wxTRANSLATE("<Select>"),
	wxTRANSLATE("Move 5 units down"),
	wxTRANSLATE("Move 5 units right"),
	wxTRANSLATE("Rotate 90\370 clockwise at (1, 2)"),
	wxTRANSLATE("Rotate 90\370 counterclockwise at (-1, 2)"),
	wxTRANSLATE("Rotate 180\370 at (0, 0)"),
	wxTRANSLATE("Flip horizontally at X = 4"),
	wxTRANSLATE("Flip vertically at Y = 3"),
	wxTRANSLATE("Scale up horizontally by a factor of 2"),
	wxTRANSLATE("Scale up vertically by a factor of 3")
};

int ASSDrawTransformDlg::combo_templatesCount = 10;

EightDouble ASSDrawTransformDlg::combo_templatesValues[] = {
	{ 1.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f }, //<Select>
	{ 1.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 5.0f }, //5 units down
	{ 1.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 5.0f, 0.0f }, //5 units left
	{ 0.0f, -1.0f, 1.0f, 0.0f, 1.0f, 2.0f, 1.0f, 2.0f }, //90 CW (1,2)
	{ 0.0f, 1.0f, -1.0f, 0.0f, -1.0f, 2.0f, -1.0f, 2.0f }, //90 CCW, (-1,2)
	{ -1.0f, 0.0f, 0.0f, -1.0f, 0.0f, 0.0f, 0.0f, 0.0f }, //180 (0,0)
	{ -1.0f, 0.0f, 0.0f, 1.0f, 4.0f, 0.0f, 4.0f, 0.0f }, //Flip X = 4
	{ 1.0f, 0.0f, 0.0f, -1.0f, 0.0f, 3.0f, 0.0f, 3.0f }, //Flip Y = 3
	{ 2.0f, 0.0f, 0.0f, 1.0f, 0.0f, 0.0f, 0.0f, 0.0f }, //Scale X * 2
	{ 1.0f, 0.0f, 0.0f, 3.0f, 0.0f, 0.0f, 0.0f, 0.0f } //Scale Y * 3
};
