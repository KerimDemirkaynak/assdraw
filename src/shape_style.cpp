// ASSDraw shape style model and ASS override-tag serialization.
// New code in this file is licensed under the BSD 3-Clause License; see LICENSE.
#include "shape_style.hpp"

#include <algorithm>
#include <cctype>
#include <cstdio>
#include <cstdlib>
#include <cmath>
#include <iomanip>
#include <locale>
#include <sstream>
#include <utility>

namespace {

std::size_t FindHexMarker(const std::string& text, const std::string& tag)
{
    const std::string marker = "\\" + tag + "&H";
    std::size_t start = text.find(marker);
    if (start != std::string::npos)
        return start;
    std::string lowercase_marker = marker;
    lowercase_marker.back() = 'h';
    return text.find(lowercase_marker);
}

bool ReadHexByte(const std::string& text, const std::string& tag, std::uint8_t& value)
{
    const std::string marker = "\\" + tag + "&H";
    const std::size_t start = FindHexMarker(text, tag);
    if (start == std::string::npos)
        return false;

    const std::size_t digits = start + marker.size();
    const std::size_t end = text.find('&', digits);
    if (end == std::string::npos || end - digits != 2)
        return false;

    const std::string digits_text = text.substr(digits, 2);
    char* parsed_end = nullptr;
    const unsigned long parsed = std::strtoul(digits_text.c_str(), &parsed_end, 16);
    if (parsed_end == nullptr || *parsed_end != '\0' || parsed > 0xff)
        return false;
    value = static_cast<std::uint8_t>(parsed);
    return true;
}

bool ReadColor(const std::string& text, const std::string& tag, AssRgb& color)
{
    const std::string marker = "\\" + tag + "&H";
    const std::size_t start = FindHexMarker(text, tag);
    if (start == std::string::npos)
        return false;

    const std::size_t digits = start + marker.size();
    const std::size_t end = text.find('&', digits);
    if (end == std::string::npos || end - digits != 6)
        return false;

    const std::string digits_text = text.substr(digits, 6);
    char* parsed_end = nullptr;
    const unsigned long parsed = std::strtoul(digits_text.c_str(), &parsed_end, 16);
    if (parsed_end == nullptr || *parsed_end != '\0' || parsed > 0xffffff)
        return false;
    color.blue = static_cast<std::uint8_t>((parsed >> 16) & 0xff);
    color.green = static_cast<std::uint8_t>((parsed >> 8) & 0xff);
    color.red = static_cast<std::uint8_t>(parsed & 0xff);
    return true;
}

} // namespace

std::string AssColorFromRgb(AssRgb color)
{
    char value[10] = {};
    std::snprintf(value, sizeof(value), "&H%02X%02X%02X&",
        static_cast<unsigned>(color.blue), static_cast<unsigned>(color.green), static_cast<unsigned>(color.red));
    return value;
}

std::string AssAlphaFromOpacity(std::uint8_t opacity)
{
    char value[6] = {};
    std::snprintf(value, sizeof(value), "&H%02X&", 255U - static_cast<unsigned>(opacity));
    return value;
}

std::string CanonicalizeAssHexLiterals(std::string text)
{
    for (std::size_t start = 0; start + 2 < text.size(); ++start) {
        if (text[start] != '&' || (text[start + 1] != 'H' && text[start + 1] != 'h'))
            continue;
        const std::size_t end = text.find('&', start + 2);
        if (end == std::string::npos || end == start + 2)
            continue;
        bool hexadecimal = true;
        for (std::size_t index = start + 2; index < end; ++index)
            hexadecimal = hexadecimal && std::isxdigit(static_cast<unsigned char>(text[index])) != 0;
        if (!hexadecimal)
            continue;
        text[start + 1] = 'H';
        for (std::size_t index = start + 2; index < end; ++index)
            text[index] = static_cast<char>(std::toupper(static_cast<unsigned char>(text[index])));
        start = end;
    }
    return text;
}

std::string ShapeStyle::SerializeOverrideTags() const
{
    std::ostringstream border;
    border.imbue(std::locale::classic());
    border << std::fixed << std::setprecision(2)
           << (std::isfinite(outline_width) ? std::max(0.0, outline_width) : 0.0);
    std::string tags = "{\\p1\\bord";
    tags += outline_enabled ? border.str() : "0";
    tags += "\\1c" + AssColorFromRgb(fill_color);
    tags += "\\1a" + AssAlphaFromOpacity(fill_enabled ? fill_opacity : 0);
    tags += "\\3c" + AssColorFromRgb(outline_color);
    tags += "\\3a" + AssAlphaFromOpacity(outline_enabled ? outline_opacity : 0);
    tags += "}";
    return tags;
}

std::string FormatAssCommandCase(std::string text, bool capitalize_drawing)
{
    bool override_block = false;
    for (char& value : text) {
        if (value == '{')
            override_block = true;
        else if (value == '}')
            override_block = false;
        else if (!override_block) {
            const unsigned char ascii = static_cast<unsigned char>(value);
            value = static_cast<char>(capitalize_drawing ? std::toupper(ascii) : std::tolower(ascii));
        }
    }
    return CanonicalizeAssHexLiterals(std::move(text));
}

bool ShapeStyle::ImportOverrideTags(const std::string& text)
{
    bool changed = false;
    changed = ReadColor(text, "1c", fill_color) || changed;
    changed = ReadColor(text, "3c", outline_color) || changed;

    std::uint8_t alpha = 0;
    if (ReadHexByte(text, "1a", alpha)) {
        fill_opacity = static_cast<std::uint8_t>(255U - alpha);
        fill_enabled = alpha != 255;
        changed = true;
    }
    if (ReadHexByte(text, "3a", alpha)) {
        outline_opacity = static_cast<std::uint8_t>(255U - alpha);
        changed = true;
    }

    const std::string marker = "\\bord";
    const std::size_t start = text.find(marker);
    if (start != std::string::npos) {
        std::istringstream input(text.substr(start + marker.size()));
        input.imbue(std::locale::classic());
        double width = 0.0;
        if ((input >> width) && std::isfinite(width)) {
            outline_width = std::max(0.0, width);
            outline_enabled = outline_width > 0.0 && outline_opacity > 0;
            changed = true;
        }
    }
    return changed;
}
