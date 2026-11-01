// ASSDraw shape style model and ASS override-tag serialization.
// New code in this file is licensed under the BSD 3-Clause License; see LICENSE.
#pragma once

#include <cstdint>
#include <string>

struct AssRgb
{
    std::uint8_t red = 0;
    std::uint8_t green = 102;
    std::uint8_t blue = 255;
};

// Opacity uses the UI convention: 255 is opaque and 0 is transparent.
struct ShapeStyle
{
    bool fill_enabled = true;
    AssRgb fill_color;
    std::uint8_t fill_opacity = 255;

    bool outline_enabled = false;
    double outline_width = 1.0;
    AssRgb outline_color { 0, 0, 0 };
    std::uint8_t outline_opacity = 255;

    std::string SerializeOverrideTags() const;
    bool ImportOverrideTags(const std::string& text);
};

std::string AssColorFromRgb(AssRgb color);
std::string AssAlphaFromOpacity(std::uint8_t opacity);
std::string CanonicalizeAssHexLiterals(std::string text);
std::string FormatAssCommandCase(std::string text, bool capitalize_drawing);
