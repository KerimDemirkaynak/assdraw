#include "drawing_history.hpp"
#include "shape_style.hpp"
#include "styled_ass.hpp"

#include <cassert>
#include <cstddef>
#include <initializer_list>
#include <locale>
#include <string>
#include <utility>
#include <vector>

struct DrawingSnapshot { std::string geometry; };
struct ReferenceImageState {
    int x;
    int y;
    double scale;
    int opacity;
};

namespace {

ShapeStyle Colored(std::uint8_t red, std::uint8_t green, std::uint8_t blue)
{
    ShapeStyle style;
    style.fill_color = { red, green, blue };
    return style;
}

AssDrawingCommand Command(char type, std::initializer_list<AssDrawingPoint> points,
    bool close_spline = false)
{
    AssDrawingCommand command;
    command.type = type;
    command.points = points;
    command.close_spline = close_spline;
    return command;
}

std::vector<AssDrawingCommand> Rectangle(std::int64_t left, std::int64_t top,
    std::int64_t right, std::int64_t bottom)
{
    return {
        Command('m', {{ left, top }}),
        Command('l', {{ right, top }}),
        Command('l', {{ right, bottom }}),
        Command('l', {{ left, bottom }})
    };
}

StyledAssSubshape Shape(const ShapeStyle& style, std::vector<AssDrawingCommand> commands)
{
    StyledAssSubshape shape;
    shape.style = style;
    shape.commands = std::move(commands);
    return shape;
}

std::size_t Count(const std::string& text, const std::string& needle)
{
    std::size_t count = 0;
    for (std::size_t position = 0; (position = text.find(needle, position)) != std::string::npos;
        position += needle.size())
        ++count;
    return count;
}

std::size_t CountCommands(const StyledAssRun& run, char type)
{
    std::size_t count = 0;
    for (const AssDrawingCommand& command : run.commands)
        if (command.type == type)
            ++count;
    return count;
}

void AssertCommandsEqual(const std::vector<AssDrawingCommand>& left,
    const std::vector<AssDrawingCommand>& right)
{
    assert(left.size() == right.size());
    for (std::size_t command_index = 0; command_index < left.size(); ++command_index) {
        assert(left[command_index].type == right[command_index].type);
        assert(left[command_index].close_spline == right[command_index].close_spline);
        assert(left[command_index].points.size() == right[command_index].points.size());
        for (std::size_t point_index = 0; point_index < left[command_index].points.size(); ++point_index) {
            assert(left[command_index].points[point_index].x == right[command_index].points[point_index].x);
            assert(left[command_index].points[point_index].y == right[command_index].points[point_index].y);
        }
    }
}

void TestColorCase()
{
    assert(CanonicalizeAssHexLiterals("&hff80ff&") == "&HFF80FF&");
    assert(CanonicalizeAssHexLiterals("\\1c&hFf80fF&\\1a&h7f&") ==
        "\\1c&HFF80FF&\\1a&H7F&");
    assert(CanonicalizeAssHexLiterals("&hnot-hex&") == "&hnot-hex&");

    const char* inputs[] = {
        "{\\bord1\\1c&hff00aa&\\1a&h7f&\\3c&h1234ab&\\3a&h2e&}",
        "{\\bord1\\1c&Hff00aa&\\1a&H7f&\\3c&H1234ab&\\3a&H2e&}",
        "{\\bord1\\1c&HFF00AA&\\1a&H7F&\\3c&H1234AB&\\3a&H2E&}"
    };
    for (const char* input : inputs) {
        ShapeStyle imported;
        assert(imported.ImportOverrideTags(input));
        assert(imported.fill_color.red == 0xAA);
        assert(imported.fill_color.green == 0x00);
        assert(imported.fill_color.blue == 0xFF);
        assert(imported.fill_opacity == 0x80);
        assert(imported.outline_color.red == 0xAB);
        assert(imported.outline_color.green == 0x34);
        assert(imported.outline_color.blue == 0x12);
        assert(imported.outline_opacity == 0xD1);
        const std::string canonical = imported.SerializeOverrideTags();
        assert(canonical.find("\\1c&HFF00AA&") != std::string::npos);
        assert(canonical.find("\\1a&H7F&") != std::string::npos);
        assert(canonical.find("\\3c&H1234AB&") != std::string::npos);
        assert(canonical.find("\\3a&H2E&") != std::string::npos);
        assert(canonical.find("&h") == std::string::npos);
    }
}

void TestSingleAndSameStyleRuns()
{
    const ShapeStyle red = Colored(255, 0, 0);
    const std::string single = SerializeStyledAss({ Shape(red, Rectangle(100, 100, 200, 200)) });
    assert(Count(single, "{\\p1") == 1);
    assert(single.find("m 100 100 l 200 100 l 200 200 l 100 200") != std::string::npos);
    assert(single.find('\r') == std::string::npos && single.find('\n') == std::string::npos);
    assert(single.find("\\1c&H0000FF&") != std::string::npos);

    const std::string same = SerializeStyledAss({
        Shape(red, Rectangle(100, 100, 200, 200)),
        Shape(red, Rectangle(400, 100, 500, 200))
    });
    assert(Count(same, "{\\p1") == 1);
    assert(Count(same, " m ") == 1);
    assert(same.find("m 400 100 l 500 100") != std::string::npos);
}

void TestDifferentStyleCompensation()
{
    const ShapeStyle red = Colored(255, 0, 0);
    const ShapeStyle blue = Colored(0, 0, 255);
    const ShapeStyle green = Colored(0, 255, 0);
    const std::string two = SerializeStyledAss({
        Shape(red, Rectangle(100, 0, 200, 100)),
        Shape(blue, Rectangle(400, 0, 500, 100))
    });
    assert(two.find("m 100 0 l 200 0") != std::string::npos);
    assert(two.find("m 300 0 l 400 0") != std::string::npos);

    const std::string three = SerializeStyledAss({
        Shape(red, Rectangle(100, 0, 200, 100)),
        Shape(blue, Rectangle(500, 0, 650, 100)),
        Shape(green, Rectangle(-100, 0, 50, 100))
    });
    assert(three.find("m 100 0 l 200 0") != std::string::npos);
    assert(three.find("m 400 0 l 550 0") != std::string::npos);
    assert(three.find("m -350 0 l -200 0") != std::string::npos);

    // The first run starts at x=100: its advance is 200-100, not x_max.
    const std::string nonzero_minimum = SerializeStyledAss({
        Shape(red, Rectangle(100, 0, 200, 100)),
        Shape(blue, Rectangle(300, 0, 400, 100))
    });
    assert(nonzero_minimum.find("m 200 0 l 300 0") != std::string::npos);
}

void TestBezierBoundsAndBorderIndependence()
{
    ShapeStyle red = Colored(255, 0, 0);
    ShapeStyle blue = Colored(0, 0, 255);
    red.outline_enabled = true;
    red.outline_width = 40.0;
    const StyledAssSubshape bezier = Shape(red, {
        Command('m', {{ 100, 0 }}),
        Command('b', {{ 50, 10 }, { 250, 20 }, { 200, 30 }})
    });
    const std::vector<StyledAssRun> runs = BuildStyledAssRuns({ bezier });
    assert(runs.size() == 1);
    assert(DrawingRunAdvance(runs.front()) == 200);

    const std::string output = SerializeStyledAss({
        bezier,
        Shape(blue, Rectangle(400, 0, 500, 100))
    });
    assert(output.find("m 200 0 l 300 0") != std::string::npos);

    red.outline_width = 1.0;
    const std::vector<StyledAssRun> thin_runs = BuildStyledAssRuns({ Shape(red, bezier.commands) });
    assert(DrawingRunAdvance(thin_runs.front()) == 200);

    const StyledAssSubshape spline = Shape(red, {
        Command('m', {{ 100, 0 }}),
        Command('s', {{ -75, 10 }, { 275, 20 }, { 200, 30 }}, true)
    });
    const std::vector<StyledAssRun> spline_runs = BuildStyledAssRuns({ spline });
    assert(DrawingRunAdvance(spline_runs.front()) == 350);
    const StyledAssImport closed_spline = ImportStyledAss(SerializeStyledAss({ spline }), ShapeStyle());
    assert(closed_spline.logical_runs.size() == 1);
    assert(closed_spline.logical_runs.front().commands.back().close_spline);
}

void TestHolesAndAdjacentMerging()
{
    const ShapeStyle red = Colored(255, 0, 0);
    const ShapeStyle blue = Colored(0, 0, 255);
    const ShapeStyle green = Colored(0, 255, 0);
    std::vector<AssDrawingCommand> outer_and_hole = Rectangle(0, 0, 300, 300);
    const std::vector<AssDrawingCommand> hole = {
        Command('m', {{ 100, 100 }}), Command('l', {{ 100, 200 }}),
        Command('l', {{ 200, 200 }}), Command('l', {{ 200, 100 }})
    };
    outer_and_hole.insert(outer_and_hole.end(), hole.begin(), hole.end());
    const std::vector<StyledAssRun> hole_runs = BuildStyledAssRuns({ Shape(red, outer_and_hole) });
    assert(hole_runs.size() == 1);
    assert(CountCommands(hole_runs.front(), 'm') == 2);
    assert(DrawingRunAdvance(hole_runs.front()) == 300);

    const std::vector<StyledAssSubshape> adjacent = {
        Shape(red, Rectangle(100, 0, 200, 100)),
        Shape(red, Rectangle(400, 0, 500, 100)),
        Shape(blue, Rectangle(800, 0, 900, 100)),
        Shape(blue, Rectangle(1000, 0, 1100, 100)),
        Shape(green, Rectangle(1500, 0, 1600, 100))
    };
    const std::vector<StyledAssRun> merged = BuildStyledAssRuns(adjacent);
    assert(merged.size() == 3);
    assert(CountCommands(merged[0], 'm') == 2);
    assert(CountCommands(merged[1], 'm') == 2);
    const std::string output = SerializeStyledAss(adjacent);
    assert(Count(output, "{\\p1") == 3);
    // Red's merged cbox is 100..500 (advance 400); blue's is 800..1100
    // (advance 300), so green is shifted by the cumulative 700.
    assert(output.find("m 400 0 l 500 0") != std::string::npos);
    assert(output.find("m 800 0 l 900 0") != std::string::npos);

    const std::vector<StyledAssRun> nonadjacent = BuildStyledAssRuns({
        Shape(red, Rectangle(0, 0, 10, 10)),
        Shape(blue, Rectangle(20, 0, 30, 10)),
        Shape(red, Rectangle(40, 0, 50, 10))
    });
    assert(nonadjacent.size() == 3);
    assert(ShapeStylesEqual(nonadjacent[0].style, red));
    assert(ShapeStylesEqual(nonadjacent[1].style, blue));
    assert(ShapeStylesEqual(nonadjacent[2].style, red));
}

void TestStyledRoundTrip()
{
    ShapeStyle red = Colored(255, 0, 0);
    red.fill_opacity = 0x80;
    red.outline_enabled = true;
    red.outline_width = 3.5;
    red.outline_color = { 20, 30, 40 };
    red.outline_opacity = 0xC0;
    const ShapeStyle blue = Colored(0, 0, 255);
    const ShapeStyle green = Colored(0, 255, 0);

    std::vector<AssDrawingCommand> outer_and_hole = Rectangle(100, 100, 300, 300);
    const std::vector<AssDrawingCommand> hole = {
        Command('m', {{ 150, 150 }}), Command('l', {{ 150, 250 }}),
        Command('l', {{ 250, 250 }}), Command('l', {{ 250, 150 }})
    };
    outer_and_hole.insert(outer_and_hole.end(), hole.begin(), hole.end());
    const std::vector<StyledAssSubshape> original = {
        Shape(red, outer_and_hole),
        Shape(blue, {
            Command('m', {{ 500, 0 }}),
            Command('b', {{ 450, 20 }, { 700, 40 }, { 650, 60 }})
        }),
        Shape(green, Rectangle(-100, -50, 50, 50))
    };

    const std::vector<StyledAssRun> original_runs = BuildStyledAssRuns(original);
    const std::string serialized = SerializeStyledAss(original);
    const StyledAssImport imported = ImportStyledAss(serialized, ShapeStyle());
    assert(imported.decompensated);
    assert(imported.logical_runs.size() == original_runs.size());
    for (std::size_t index = 0; index < original_runs.size(); ++index) {
        assert(ShapeStylesEqual(imported.logical_runs[index].style, original_runs[index].style));
        AssertCommandsEqual(imported.logical_runs[index].commands, original_runs[index].commands);
    }
    assert(imported.contour_styles.size() == 4);
    assert(ShapeStylesEqual(imported.contour_styles[0], red));
    assert(ShapeStylesEqual(imported.contour_styles[1], red));
    assert(ShapeStylesEqual(imported.contour_styles[2], blue));
    assert(ShapeStylesEqual(imported.contour_styles[3], green));
    assert(CountCommands(imported.logical_runs[0], 'm') == 2);

    // Plain historical drawing text has no style-separated runs and must not
    // receive inverse inline-layout compensation. Multiline whitespace is OK.
    const StyledAssImport plain = ImportStyledAss(
        "m 100 0 l 200 0\r\n m 400 0 l 500 0", red);
    assert(!plain.decompensated);
    assert(plain.drawing_text.find("m 400 0 l 500 0") != std::string::npos);
}

} // namespace

int main()
{
    struct CommaDecimal : std::numpunct<char> {
        char do_decimal_point() const override { return ','; }
    };
    const std::locale previous = std::locale();
    std::locale::global(std::locale(previous, new CommaDecimal));
    ShapeStyle localized;
    localized.outline_enabled = true;
    localized.outline_width = 2.5;
    assert(localized.SerializeOverrideTags().find("\\bord2.50") != std::string::npos);
    assert(localized.ImportOverrideTags("{\\bord3.25}"));
    assert(localized.outline_width == 3.25);
    std::locale::global(previous);

    assert(FormatAssCommandCase("{\\p1\\bord1\\1c&hff00aa&}m 0 0 l 5 5", true) ==
        "{\\p1\\bord1\\1c&HFF00AA&}M 0 0 L 5 5");
    assert(FormatAssCommandCase("{\\p1\\bord1\\1c&HFF00AA&}M 0 0 L 5 5", false) ==
        "{\\p1\\bord1\\1c&HFF00AA&}m 0 0 l 5 5");

    assert(AssColorFromRgb({255, 0, 0}) == "&H0000FF&");
    assert(AssColorFromRgb({0, 255, 0}) == "&H00FF00&");
    assert(AssColorFromRgb({0, 0, 255}) == "&HFF0000&");
    assert(AssAlphaFromOpacity(255) == "&H00&");
    assert(AssAlphaFromOpacity(0) == "&HFF&");

    ShapeStyle style;
    style.fill_color = { 0xF5, 0xA5, 0x42 };
    style.outline_enabled = true;
    style.outline_width = 3;
    assert(style.SerializeOverrideTags() == "{\\p1\\bord3.00\\1c&H42A5F5&\\1a&H00&\\3c&H000000&\\3a&H00&}");

    TestColorCase();
    TestSingleAndSameStyleRuns();
    TestDifferentStyleCompensation();
    TestBezierBoundsAndBorderIndependence();
    TestHolesAndAdjacentMerging();
    TestStyledRoundTrip();

    DrawingHistory<DrawingSnapshot> history;
    const ReferenceImageState background { 47, -19, 1.75, 63 };
    DrawingSnapshot drawing { "m 0 0 l 10 0" };
    history.Push(drawing);
    drawing.geometry = "m 0 0 l 10 0 l 10 10";
    history.Push(drawing);
    drawing.geometry = "m 0 0 l 10 0 l 10 10 l 0 10";

    DrawingSnapshot restored;
    assert(history.Undo(drawing, restored));
    drawing = restored;
    assert(drawing.geometry == "m 0 0 l 10 0 l 10 10");
    assert(background.x == 47 && background.y == -19 && background.scale == 1.75 && background.opacity == 63);
    assert(history.Undo(drawing, restored));
    drawing = restored;
    assert(drawing.geometry == "m 0 0 l 10 0");
    assert(background.x == 47 && background.y == -19 && background.scale == 1.75 && background.opacity == 63);
    assert(history.Redo(drawing, restored));
    drawing = restored;
    assert(drawing.geometry == "m 0 0 l 10 0 l 10 10");
    assert(background.x == 47 && background.y == -19 && background.scale == 1.75 && background.opacity == 63);
}
