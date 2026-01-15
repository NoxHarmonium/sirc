#ifndef CONSTANTS_HPP
#define CONSTANTS_HPP
#include <cstdint>

constexpr int WIDTH_PIXELS = 256;
constexpr int HEIGHT_PIXELS = 256;
constexpr int TOTAL_PIXELS = WIDTH_PIXELS * HEIGHT_PIXELS;

constexpr int WIDTH_TILEMAP = 32;
constexpr int HEIGHT_TILEMAP = 32;
constexpr int TOTAL_TILES = WIDTH_TILEMAP * HEIGHT_TILEMAP;

// The 'transparent' colour is always at the first index of the palette
// It can technically be any colour (e.g. bright pink), but at the
// moment we are standardising on black.
// Anything that is actually black will need to be a very dark gray
constexpr uint16_t TRANSPARENCY_COLOR = 0x0000;

#endif // CONSTANTS_HPP
