// Styled ASS drawing-run serialization.
// New code in this file is licensed under the BSD 3-Clause License; see LICENSE.
#include "styled_ass.hpp"

#include <algorithm>
#include <cctype>
#include <cstdlib>
#include <limits>
#include <sstream>
#include <utility>

namespace {

bool IsDrawingCommand(const std::string& token)
{
    if (token.size() != 1)
        return false;
    switch (static_cast<char>(std::tolower(static_cast<unsigned char>(token[0])))) {
        case 'm': case 'n': case 'l': case 'b': case 's': case 'p': case 'c':
            return true;
        default:
            return false;
    }
}

bool ReadInteger(const std::string& token, std::int64_t& value)
{
    if (token.empty())
        return false;
    char* end = nullptr;
    const long long parsed = std::strtoll(token.c_str(), &end, 10);
    if (end == token.c_str() || *end != '\0')
        return false;
    value = static_cast<std::int64_t>(parsed);
    return true;
}

std::vector<AssDrawingCommand> ParseDrawingCommands(const std::string& text)
{
    std::vector<AssDrawingCommand> commands;
    std::istringstream stream(text);
    std::string token;
    AssDrawingCommand current;
    bool have_command = false;
    std::vector<std::int64_t> coordinates;

    auto flush = [&]() {
        if (!have_command)
            return;
        if (current.type == 'c') {
            commands.push_back(current);
        }
        else {
            for (std::size_t index = 0; index + 1 < coordinates.size(); index += 2)
                current.points.push_back({ coordinates[index], coordinates[index + 1] });
            if (!current.points.empty())
                commands.push_back(current);
        }
        current = AssDrawingCommand();
        coordinates.clear();
        have_command = false;
    };

    while (stream >> token) {
        if (IsDrawingCommand(token)) {
            flush();
            const char type = static_cast<char>(std::tolower(static_cast<unsigned char>(token[0])));
            if (type == 'c' && !commands.empty() &&
                (commands.back().type == 's' || commands.back().type == 'p')) {
                commands.back().close_spline = true;
                continue;
            }
            current.type = type;
            have_command = true;
            continue;
        }
        std::int64_t coordinate = 0;
        if (have_command && ReadInteger(token, coordinate))
            coordinates.push_back(coordinate);
    }
    flush();
    return commands;
}

std::string SerializeCommands(const std::vector<AssDrawingCommand>& commands, std::int64_t x_offset)
{
    std::ostringstream output;
    bool first_command = true;
    for (const AssDrawingCommand& command : commands) {
        if (!first_command)
            output << ' ';
        first_command = false;
        output << command.type;
        for (std::size_t index = 0; index < command.unknown_point_pairs; ++index)
            output << " ? ?";
        for (const AssDrawingPoint& point : command.points)
            output << ' ' << point.x + x_offset << ' ' << point.y;
        if (command.close_spline)
            output << " c";
    }
    return output.str();
}

void TranslateRunX(StyledAssRun& run, std::int64_t x_offset)
{
    for (AssDrawingCommand& command : run.commands)
        for (AssDrawingPoint& point : command.points)
            point.x += x_offset;
}

struct ImportedRun
{
    StyledAssRun run;
    bool has_style_override = false;
};

bool AddImportedRun(std::vector<ImportedRun>& runs, const std::string& drawing,
    const ShapeStyle& style, bool has_style_override)
{
    std::vector<AssDrawingCommand> commands = ParseDrawingCommands(drawing);
    if (commands.empty())
        return false;
    ImportedRun imported;
    imported.run.style = style;
    imported.run.commands = std::move(commands);
    imported.has_style_override = has_style_override;
    runs.push_back(std::move(imported));
    return true;
}

} // namespace

bool ShapeStylesEqual(const ShapeStyle& left, const ShapeStyle& right)
{
    // Serialization equality is effective-style equality. In particular, two
    // disabled outlines both emit bord0 even if the UI remembers different
    // widths for a later re-enable operation.
    return left.SerializeOverrideTags() == right.SerializeOverrideTags();
}

std::vector<StyledAssRun> BuildStyledAssRuns(const std::vector<StyledAssSubshape>& subshapes)
{
    std::vector<StyledAssRun> runs;
    for (const StyledAssSubshape& subshape : subshapes) {
        if (subshape.commands.empty())
            continue;
        if (!runs.empty() && ShapeStylesEqual(runs.back().style, subshape.style)) {
            runs.back().commands.insert(runs.back().commands.end(),
                subshape.commands.begin(), subshape.commands.end());
            continue;
        }
        StyledAssRun run;
        run.style = subshape.style;
        run.commands = subshape.commands;
        runs.push_back(std::move(run));
    }
    return runs;
}

std::int64_t DrawingRunAdvance(const StyledAssRun& run)
{
    std::int64_t minimum = std::numeric_limits<std::int64_t>::max();
    std::int64_t maximum = std::numeric_limits<std::int64_t>::min();
    for (const AssDrawingCommand& command : run.commands) {
        for (const AssDrawingPoint& point : command.points) {
            minimum = std::min(minimum, point.x);
            maximum = std::max(maximum, point.x);
        }
    }
    return minimum == std::numeric_limits<std::int64_t>::max() ? 0 : maximum - minimum;
}

std::string SerializeStyledAss(const std::vector<StyledAssSubshape>& subshapes)
{
    const std::vector<StyledAssRun> runs = BuildStyledAssRuns(subshapes);
    std::ostringstream output;
    std::int64_t cumulative_previous_advance = 0;
    for (std::size_t index = 0; index < runs.size(); ++index) {
        if (index != 0)
            output << ' ';
        output << runs[index].style.SerializeOverrideTags();
        output << SerializeCommands(runs[index].commands, -cumulative_previous_advance);
        cumulative_previous_advance += DrawingRunAdvance(runs[index]);
    }
    return output.str();
}

StyledAssImport ImportStyledAss(const std::string& source, const ShapeStyle& initial_style)
{
    std::vector<ImportedRun> imported_runs;
    ShapeStyle active_style = initial_style;
    bool pending_style_override = false;
    std::size_t segment_start = 0;

    for (std::size_t index = 0; index < source.size();) {
        if (source[index] != '{') {
            ++index;
            continue;
        }
        if (AddImportedRun(imported_runs, source.substr(segment_start, index - segment_start),
            active_style, pending_style_override))
            pending_style_override = false;
        const std::size_t end = source.find('}', index + 1);
        if (end == std::string::npos)
            break;
        ShapeStyle candidate = active_style;
        const bool style_override = candidate.ImportOverrideTags(source.substr(index, end - index + 1));
        if (style_override)
            active_style = candidate;
        pending_style_override = pending_style_override || style_override;
        index = end + 1;
        segment_start = index;
    }
    AddImportedRun(imported_runs, source.substr(segment_start), active_style, pending_style_override);

    StyledAssImport result;
    if (imported_runs.empty()) {
        result.drawing_text = source;
        return result;
    }

    result.decompensated = imported_runs.size() > 1;
    for (const ImportedRun& imported : imported_runs)
        result.decompensated = result.decompensated && imported.has_style_override;

    std::int64_t cumulative_previous_advance = 0;
    for (ImportedRun& imported : imported_runs) {
        const std::int64_t advance = DrawingRunAdvance(imported.run);
        if (result.decompensated)
            TranslateRunX(imported.run, cumulative_previous_advance);
        cumulative_previous_advance += advance;

        for (const AssDrawingCommand& command : imported.run.commands) {
            if (command.type == 'm')
                result.contour_styles.push_back(imported.run.style);
        }
        result.logical_runs.push_back(std::move(imported.run));
    }

    std::ostringstream drawing;
    for (std::size_t index = 0; index < result.logical_runs.size(); ++index) {
        if (index != 0)
            drawing << ' ';
        drawing << SerializeCommands(result.logical_runs[index].commands, 0);
    }
    result.drawing_text = drawing.str();
    return result;
}
