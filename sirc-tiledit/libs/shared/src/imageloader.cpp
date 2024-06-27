
#include <imageloader.hpp>

#include <png.h>

#include <cassert>
#include <cstdlib>

// Thanks to https://gist.github.com/niw/5963798 for the original code for this
// file

//NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays,modernize-avoid-c-arrays)
constexpr png_byte emptyPixel[] = {0, 0, 0, 0xFF};
const auto *const emptyPixelPtr =
    static_cast<const png_byte *const>(emptyPixel);

RgbaPixelData ImageLoader::loadImageFromPng(const char *filename) {
  RgbaPixelData output = {};

  FILE *fp = fopen(filename, "rb");

  // ====
  // This section is just initialisation and housekeeping

  auto *png =
      png_create_read_struct(PNG_LIBPNG_VER_STRING, nullptr, nullptr, nullptr);

  assert(png != nullptr);

  auto *info = png_create_info_struct(png);

  assert(info != nullptr);

  auto cleanup = [&png, &info, &fp] {
    png_destroy_read_struct(&png, &info, nullptr);
    fclose(fp);
  };

  // NOLINTNEXTLINE(cert-err52-cpp, cppcoreguidelines-pro-bounds-array-to-pointer-decay)
  if (setjmp(png_jmpbuf(png))) {
    // libpng will jump here if an internal error occurs
    // This cleans up the data structures, which is good practice but not
    // really needed because of the abort below
    cleanup();
    // We will just abort for now, but we should probably have more
    // sophisticated error handling in the future
    abort();
  }

  // ====
  // This section reads the PNG file and wors out its metadata

  png_init_io(png, fp);
  png_read_info(png, info);

  const auto width = png_get_image_width(png, info);
  const auto height = png_get_image_height(png, info);
  const auto color_type = png_get_color_type(png, info);
  const auto bit_depth = png_get_bit_depth(png, info);

  // Read any color_type into 8bit depth, RGBA format.
  // See http://www.libpng.org/pub/png/libpng-manual.txt

  if (bit_depth == 16) {
    png_set_strip_16(png);
  }

  if (color_type == PNG_COLOR_TYPE_PALETTE) {
    png_set_palette_to_rgb(png);
  }

  // ====
  // This section makes sure that we get a consistent pixel format
  // even if the input files can have all sorts of formats like grayscale or
  // 16bit color depth

  // PNG_COLOR_TYPE_GRAY_ALPHA is always 8 or 16bit depth.
  if (color_type == PNG_COLOR_TYPE_GRAY && bit_depth < 8) {
    png_set_expand_gray_1_2_4_to_8(png);
  }

  if (png_get_valid(png, info, PNG_INFO_tRNS)) {
    png_set_tRNS_to_alpha(png);
  }

  // These color_type don't have an alpha channel then fill it with 0xff.
  if (color_type == PNG_COLOR_TYPE_RGB || color_type == PNG_COLOR_TYPE_GRAY ||
      color_type == PNG_COLOR_TYPE_PALETTE) {
    png_set_filler(png, 0xFF, PNG_FILLER_AFTER);
  }

  if (color_type == PNG_COLOR_TYPE_GRAY ||
      color_type == PNG_COLOR_TYPE_GRAY_ALPHA) {
    png_set_gray_to_rgb(png);
  }

  png_read_update_info(png, info);

  // ====
  // This section does the actual copying of the PNG data into memory
  // owned by us

  // It will be an uphill battle to get this c based libpng interface working
  // without malloc and pointer arithmetic
  //NOLINTBEGIN(cppcoreguidelines-no-malloc, cppcoreguidelines-pro-bounds-pointer-arithmetic, cppcoreguidelines-pro-bounds-constant-array-index)
  auto *row_pointers =
      static_cast<png_bytepp>(malloc(sizeof(png_bytep) * height));
  for (png_uint_32 y = 0; y < height; y++) {
    row_pointers[y] =
        static_cast<png_bytep>(malloc(png_get_rowbytes(png, info)));
  }

  png_read_image(png, row_pointers);

  cleanup();

  // ====
  // This section converts the format that libpng gives us into the format
  // expected by the rest of the program.
  // It will also pad/truncate the image to make sure it fits in 256x256

  // Note: image will currently be truncated or padded to fit into the tile map
  // size
  // TODO: Add image processing options for image import
  // category=tiledit
  for (size_t y = 0; y < height; y++) {
    // const png_byte * const
    // const png_byte * const

    const auto *const row = y >= HEIGHT_PIXELS ? nullptr : row_pointers[y];

    for (size_t x = 0; x < width; x++) {
      auto const *px = row == nullptr || x >= WIDTH_PIXELS ? &(emptyPixelPtr[0])
                                                           : &(row[x * 4]);
      output[x][y] =
          static_cast<uint32_t>(px[0] << 24 | px[1] << 16 | px[2] << 8 | px[3]);
    }
  }

  for (png_uint_32 y = 0; y < height; y++) {
    free(row_pointers[y]);
  }
  //NOLINTNEXTLINE(bugprone-multi-level-implicit-pointer-conversion)
  free(row_pointers);

  //NOLINTEND(cppcoreguidelines-no-malloc, cppcoreguidelines-pro-bounds-pointer-arithmetic, cppcoreguidelines-pro-bounds-constant-array-index)

  return output;
}

void ImageLoader::saveImageToPng(const char *filename,
                                 const RgbaPixelData &data) {

  FILE *fp = fopen(filename, "wb");

  // ====
  // This section is just initialisation and housekeeping

  auto *png =
      png_create_write_struct(PNG_LIBPNG_VER_STRING, nullptr, nullptr, nullptr);

  assert(png != nullptr);

  auto *info = png_create_info_struct(png);

  assert(info != nullptr);

  auto cleanup = [&png, &info, &fp] {
    png_destroy_read_struct(&png, &info, nullptr);
    fclose(fp);
  };

  // NOLINTNEXTLINE(cert-err52-cpp, cppcoreguidelines-pro-bounds-array-to-pointer-decay)
  if (setjmp(png_jmpbuf(png))) {
    // libpng will jump here if an internal error occurs
    // This cleans up the data structures, which is good practice but not
    // really needed because of the abort below
    cleanup();
    // We will just abort for now, but we should probably have more
    // sophisticated error handling in the future
    abort();
  }

  // ====
  // This section sets the PNG parameters and writes out the PNG metedata

  png_init_io(png, fp);

  // Output is 8bit depth, RGBA format.
  png_set_IHDR(png, info, WIDTH_PIXELS, HEIGHT_PIXELS, 8, PNG_COLOR_TYPE_RGBA,
               PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT,
               PNG_FILTER_TYPE_DEFAULT);
  png_write_info(png, info);

  // ====
  // This section sets up the PNG pixel data and writes it to the file

  // It will be an uphill battle to get this c based libpng interface working
  // without malloc and pointer arithmetic
  //NOLINTBEGIN(cppcoreguidelines-no-malloc, cppcoreguidelines-pro-bounds-pointer-arithmetic, cppcoreguidelines-pro-bounds-constant-array-index)
  auto *const row_pointers =
      static_cast<png_bytepp>(malloc(sizeof(png_bytep) * HEIGHT_PIXELS));

  for (png_uint_32 y = 0; y < HEIGHT_PIXELS; y++) {
    row_pointers[y] =
        static_cast<png_bytep>(malloc(sizeof(png_byte) * 4 * WIDTH_PIXELS));
  }

  for (size_t y = 0; y < HEIGHT_PIXELS; y++) {
    auto *const row = row_pointers[y];
    for (size_t x = 0; x < WIDTH_PIXELS; x++) {
      const auto pixel = data[x][y];
      auto *const bytes = &(row[x * 4]);
      bytes[0] = pixel >> 24 & 0xFF;
      bytes[1] = pixel >> 16 & 0xFF;
      bytes[2] = pixel >> 8 & 0xFF;
      bytes[3] = pixel & 0xFF;
    }
  }

  png_write_image(png, row_pointers);
  png_write_end(png, nullptr);

  cleanup();

  for (int y = 0; y < HEIGHT_PIXELS; y++) {
    free(row_pointers[y]);
  }
  //NOLINTNEXTLINE(bugprone-multi-level-implicit-pointer-conversion)
  free(row_pointers);

  //NOLINTEND(cppcoreguidelines-no-malloc, cppcoreguidelines-pro-bounds-pointer-arithmetic, cppcoreguidelines-pro-bounds-constant-array-index)
}