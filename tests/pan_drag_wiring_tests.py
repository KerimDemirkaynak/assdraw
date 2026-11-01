"""ASSDraw toolbar/menu pan wiring regression tests.

The Windows binary smoke test checks startup; these static contracts additionally
guard the command-event routes needed for background and drawing right-drag.
"""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
FRAME = (ROOT / "src" / "assdraw.cpp").read_text(encoding="utf-8-sig")
CANVAS = (ROOT / "src" / "canvas.cpp").read_text(encoding="utf-8-sig")


def body(text, start, end):
    return text.split(start, 1)[1].split(end, 1)[0]


class PanDragWiringTests(unittest.TestCase):
    def test_toolbar_and_menu_dispatch_both_pan_toggles(self):
        events = body(FRAME, "BEGIN_EVENT_TABLE(ASSDrawFrame, wxFrame)",
                      "END_EVENT_TABLE()")
        for kind in ("TOOL", "MENU"):
            self.assertRegex(events,
                rf"EVT_{kind}_RANGE\(DRAG_DWG,\s*DRAG_BGIMG,\s*"
                r"ASSDrawFrame::OnChoose_DragMode\)")

        handler = body(FRAME, "void ASSDrawFrame::OnChoose_DragMode",
                       "void ASSDrawFrame::OnZoomSliderChanged")
        self.assertIn("case DRAG_DWG: dm.drawing = !dm.drawing;", handler)
        self.assertIn("case DRAG_BGIMG: dm.bgimg = !dm.bgimg;", handler)
        self.assertIn("m_canvas->SetDragMode( dm );", handler)
        self.assertIn("UpdateFrameUI();", handler)

    def test_right_drag_moves_only_enabled_view_layers(self):
        moving = body(CANVAS, "void ASSDrawCanvas::OnMouseMove",
                      "void ASSDrawCanvas::OnMouseLeftUp")
        self.assertRegex(moving, r"if \(dragAnchor_right != NULL\)")
        self.assertIn("MoveCanvas(xx - dragAnchor_right->x, yy - dragAnchor_right->y);",
                      moving)
        self.assertIn("dragAnchor_right->x = xx;", moving)
        self.assertIn("dragAnchor_right->y = yy;", moving)
        dispatch = body(CANVAS, "void ASSDrawCanvas::MoveCanvas(",
                        "void ASSDrawCanvas::MoveCanvasDrawing")
        self.assertIn("drag_mode.drawing", dispatch)
        self.assertIn("MoveCanvasDrawing(xamount, yamount)", dispatch)
        self.assertIn("drag_mode.bgimg", dispatch)
        self.assertIn("MoveCanvasBackground(xamount, yamount)", dispatch)
        background = body(CANVAS, "void ASSDrawCanvas::MoveCanvasBackground",
                          "void ASSDrawCanvas::OnSelect_ConvertLineToBezier")
        self.assertIn("bgimg.new_disp.x += xamount;", background)
        self.assertIn("bgimg.new_disp.y += yamount;", background)
        self.assertIn("UpdateBackgroundImgScalePosition();", background)

    def test_missing_background_cannot_enable_pan(self):
        updates = body(FRAME, "void ASSDrawFrame::UpdateFrameUI",
                       "void ASSDrawFrame::OnClose")
        self.assertIn("bgimgtbar->EnableTool(DRAG_BGIMG, hasbg);", updates)
        self.assertIn("bgimgMenu->Enable(DRAG_BGIMG, hasbg);", updates)


if __name__ == "__main__":
    unittest.main()
