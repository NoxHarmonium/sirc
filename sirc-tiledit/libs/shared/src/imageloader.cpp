
#include <imageloader.hpp>

#include <png.h>

#include <cassert>

// Thanks to https://gist.github.com/niw/5963798 for libpng file io reference

RgbaPixelData ImageLoader::loadImageFromPng(const char *filename) {
  RgbaPixelData output = {};

  FILE *fp = fopen(filename, "rb");

  png_structp png =
      png_create_read_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
  if (!png)
    abort();

  png_infop info = png_create_info_struct(png);
  if (!info)
    abort();

  if (setjmp(png_jmpbuf(png)))
    abort();

  png_init_io(png, fp);

  png_read_info(png, info);

  const auto width = png_get_image_width(png, info);
  const auto height = png_get_image_height(png, info);
  const auto color_type = png_get_color_type(png, info);
  const auto bit_depth = png_get_bit_depth(png, info);

  // Read any color_type into 8bit depth, RGBA format.
  // See http://www.libpng.org/pub/png/libpng-manual.txt

  if (bit_depth == 16)
    png_set_strip_16(png);

  if (color_type == PNG_COLOR_TYPE_PALETTE)
    png_set_palette_to_rgb(png);

  // PNG_COLOR_TYPE_GRAY_ALPHA is always 8 or 16bit depth.
  if (color_type == PNG_COLOR_TYPE_GRAY && bit_depth < 8)
    png_set_expand_gray_1_2_4_to_8(png);

  if (png_get_valid(png, info, PNG_INFO_tRNS))
    png_set_tRNS_to_alpha(png);

  // These color_type don't have an alpha channel then fill it with 0xff.
  if (color_type == PNG_COLOR_TYPE_RGB || color_type == PNG_COLOR_TYPE_GRAY ||
      color_type == PNG_COLOR_TYPE_PALETTE)
    png_set_filler(png, 0xFF, PNG_FILLER_AFTER);

  if (color_type == PNG_COLOR_TYPE_GRAY ||
      color_type == PNG_COLOR_TYPE_GRAY_ALPHA)
    png_set_gray_to_rgb(png);

  png_read_update_info(png, info);

  png_bytep *row_pointers = (png_bytep *)malloc(sizeof(png_bytep) * height);
  for (png_uint_32 y = 0; y < height; y++) {
    row_pointers[y] = (png_byte *)malloc(png_get_rowbytes(png, info));
  }

  png_read_image(png, row_pointers);

  fclose(fp);

  png_destroy_read_struct(&png, &info, NULL);

  // Note: image will currently be truncated or padded to fit into the tile map
  // size
  // TODO: Add image processing options for image import
  // category=tiledit
  for (png_uint_32 y = 0; y < height; y++) {
    png_bytep row = y >= HEIGHT_PIXELS ? 0x0 : row_pointers[y];
    for (png_uint_32 x = 0; x < width; x++) {
      png_bytep px = row == 0x0 || x >= WIDTH_PIXELS ? 0x0 : &(row[x * 4]);
      // printf("%4d, %4d = RGBA(%3d, %3d, %3d, %3d)\n", x, y, px[0], px[1],
      // px[2], px[3]);
      output[x][y] =
          uint32_t((unsigned char)(px[0]) << 24 | (unsigned char)(px[1]) << 16 |
                   (unsigned char)(px[2]) << 8 | (unsigned char)(px[3]));
    }
  }

  for (png_uint_32 y = 0; y < height; y++) {
    free(row_pointers[y]);
  }
  free(row_pointers);

  fclose(fp);

  return output;
}

void ImageLoader::saveImageToPng(const char *filename,
                                 const RgbaPixelData &data) {

  FILE *fp = fopen(filename, "wb");
  if (!fp)
    abort();

  png_structp png =
      png_create_write_struct(PNG_LIBPNG_VER_STRING, NULL, NULL, NULL);
  if (!png)
    abort();

  png_infop info = png_create_info_struct(png);
  if (!info)
    abort();

  if (setjmp(png_jmpbuf(png)))
    abort();

  png_init_io(png, fp);

  // Output is 8bit depth, RGBA format.
  png_set_IHDR(png, info, WIDTH_PIXELS, HEIGHT_PIXELS, 8, PNG_COLOR_TYPE_RGBA,
               PNG_INTERLACE_NONE, PNG_COMPRESSION_TYPE_DEFAULT,
               PNG_FILTER_TYPE_DEFAULT);
  png_write_info(png, info);

  // To remove the alpha channel for PNG_COLOR_TYPE_RGB format,
  // Use png_set_filler().
  // png_set_filler(png, 0, PNG_FILLER_AFTER);
  png_bytep *row_pointers =
      (png_bytep *)malloc(sizeof(png_bytep) * HEIGHT_PIXELS);

  for (png_uint_32 y = 0; y < HEIGHT_PIXELS; y++) {
    row_pointers[y] = (png_byte *)malloc(sizeof(png_byte) * 4 * WIDTH_PIXELS);
    ;
  }

  for (int y = 0; y < HEIGHT_PIXELS; y++) {
    png_bytep row = row_pointers[y];
    for (int x = 0; x < WIDTH_PIXELS; x++) {
      RgbaPixel pixel = data[x][y];
      auto bytes = &(row[x * 4]);
      bytes[0] = pixel >> 24 & 0xFF;
      bytes[1] = pixel >> 16 & 0xFF;
      bytes[2] = pixel >> 8 & 0xFF;
      bytes[3] = pixel & 0xFF;
    }
  }

  png_write_image(png, row_pointers);
  png_write_end(png, NULL);

  for (int y = 0; y < HEIGHT_PIXELS; y++) {
    free(row_pointers[y]);
  }
  free(row_pointers);

  fclose(fp);

  png_destroy_write_struct(&png, &info);
}