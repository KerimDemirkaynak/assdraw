// Styled ASS drawing-run serialization.
// New code in this file is licensed under the BSD 3-Clause License; see LICENSE.
#pragma once

#include "shape_style.hpp"

#include <cstddef>
#include <cstdint>
#include <string>
#include <vector>

struct AssDrawingPoint
{
    std::int64_t x = 0;
    std::int64_t y = 0;
};

// A command stores every coordinate-bearing token in source order. This is
// deliberately a token-coordinate model: ASS drawing advance uses the parser
// cbox, including Bezier and spline controls, rather than rendered pixels.
struct AssDrawingCommand
{
    char type = 'm';
    std::vector<AssDrawingPoint> points;
    std::size_t unknown_point_pairs = 0;
    bool close_spline = false;
};

struct StyledAssSubshape
{
    ShapeStyle style;
    std::vector<AssDrawingCommand> commands;
};

struct StyledAssRun
{
    ShapeStyle style;
    std::vector<AssDrawingCommand> commands;
};

struct StyledAssImport
{
    std::string drawing_text;
    std::vector<ShapeStyle> contour_styles;
    std::vector<StyledAssRun> logical_runs;
    bool decompensated = false;
};

bool ShapeStylesEqual(const ShapeStyle& left, const ShapeStyle& right);
std::vector<StyledAssRun> BuildStyledAssRuns(const std::vector<StyledAssSubshape>& subshapes);
std::int64_t DrawingRunAdvance(const StyledAssRun& run);
std::string SerializeStyledAss(const std::vector<StyledAssSubshape>& subshapes);

// Multiline input remains accepted. Successive style-separated runs are moved
// back into ASSDraw's shared logical coordinate system; ordinary single-run
// drawing text is parsed without any coordinate offset.
StyledAssImport ImportStyledAss(const std::string& source, const ShapeStyle& initial_style);
