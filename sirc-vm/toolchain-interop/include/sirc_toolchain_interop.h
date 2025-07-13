#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct CTilemap {
  const uint16_t *packed_pixel_data;
  uintptr_t packed_pixel_data_len;
  uint16_t palette[16];
};


// Manual change to test CI/CD

extern "C" {

/// Takes tilemap data and returns a string that contains assembly code that will assemble to the
/// same tilemap.
///
/// Useful when you want to embed tilemap data in a program, which is probably the simplest way to
/// do it at the moment (although it might be possible to just link in a raw binary file).
char *tilemap_to_str(CTilemap tilemap);

/// Frees a string returned by one of the rust functions in this file.
///
/// Should be called when the string is no longer needed to avoid memory leaks.
///
/// # Safety
///
/// The parameter passed to this function _must_ have been allocated by one of the rust functions
/// in this file.
void free_str(char *str);

}  // extern "C"
