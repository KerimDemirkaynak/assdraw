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
const char* const TIPS_CLEAR = wxTRANSLATE("Clear the canvas and create a new drawing");
const char* const TIPS_EDITSRC = wxTRANSLATE("Edit the source");
const char* const TIPS_PREVIEW = wxTRANSLATE("Draw the shapes without enlarged points and control points");
const char* const TIPS_TRANSFORM = wxTRANSLATE("Transform the drawing using matrix transformation");
const char* const TIPS_LIBRARY = wxTRANSLATE("Shapes library");
const char* const TIPS_HELP = wxTRANSLATE("Help! Help!");
const char* const TIPS_PASTE = wxTRANSLATE("Depending on what's in the clipboard, import as drawing commands or background");
const char* const TIPS_UNDO = wxTRANSLATE("Undo last action");
const char* const TIPS_REDO = wxTRANSLATE("Redo last undo");
const char* const TIPS_ARR = wxTRANSLATE("Drag mode");
const char* const TIPS_M = wxTRANSLATE("Draw M mode (Close current shape and move the virtual pen to a new point)");
const char* const TIPS_N = wxTRANSLATE("Draw N mode (Same as M but doesn't close the shape)");
const char* const TIPS_L = wxTRANSLATE("Draw L mode (Straight line)");
const char* const TIPS_B = wxTRANSLATE("Draw B mode (Cubic Bezier curve)");
const char* const TIPS_S = wxTRANSLATE("Draw S mode (Spline)");
const char* const TIPS_P = wxTRANSLATE("Draw P mode (Extends a spline with another point)");
const char* const TIPS_C = wxTRANSLATE("Draw C mode (Close the last spline)");
const char* const TIPS_DEL = wxTRANSLATE("Delete mode");
const char* const TIPS_NUTB = wxTRANSLATE("Bilinear transformation mode: Drag the vertices to distort the shape; Dragging an edge moves two adjacent vertices together");
const char* const TIPS_SCALEROTATE = wxTRANSLATE("Scale/Rotate mode: Drag a vertex or an edge to rescale the shape; Right-drag to rotate");
const char* const TIPS_DWG = wxTRANSLATE("Right-dragging pans drawing, mousewheel zooms in/out drawing");
const char* const TIPS_BGIMG = wxTRANSLATE("Right-dragging pans background, mousewheel zooms in/out background");
const char* const TIPS_BOTH = wxTRANSLATE("Right-dragging pans drawing & background, mousewheel zooms in/out drawing & background");

const char* const TBNAME_DRAW = wxTRANSLATE("Canvas");
const char* const TBNAME_MODE = wxTRANSLATE("Drawing mode");
const char* const TBNAME_BGIMG = wxTRANSLATE("Background");
const char* const TBNAME_ZOOM = wxTRANSLATE("Zoom");

const char* const ASSDrawTransformDlg::combo_templatesStrings[] = {
	wxTRANSLATE("<Select>"),
	wxTRANSLATE("Move 5 units down"),
	wxTRANSLATE("Move 5 units right"),
	wxTRANSLATE("Rotate 90° clockwise at (1, 2)"),
	wxTRANSLATE("Rotate 90° counterclockwise at (-1, 2)"),
	wxTRANSLATE("Rotate 180° at (0, 0)"),
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
