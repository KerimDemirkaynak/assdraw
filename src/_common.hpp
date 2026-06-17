
#include "wx/wxprec.h"
#include <wx/bmpbndl.h>

#ifdef __BORLANDC__
    #pragma hdrstop
#endif

#ifndef WX_PRECOMP
    #include "wx/wx.h"
#endif

#define TITLE wxT("ASSDraw3")

#define __DPDS__ wxDefaultPosition, wxDefaultSize
#define ASSDRAW_BITMAP_BUNDLE(name) wxBitmapBundle::FromBitmap(wxBITMAP(name))
