#include "test_utils.hpp"

#include <algorithm>
#include <iterator>
#include <string>

bool compare_files(const std::string &filename1, const std::string &filename2) {
  std::ifstream file1(filename1, std::ios::binary);
  std::ifstream file2(filename2, std::ios::binary);

  if (!file1) {
    throw std::runtime_error("Failed to open file: " + filename1);
  }
  if (!file2) {
    throw std::runtime_error("Failed to open file: " + filename2);
  }

  return std::equal(std::istreambuf_iterator<char>(file1),
                    std::istreambuf_iterator<char>(),
                    std::istreambuf_iterator<char>(file2));
}