#include "utils.hpp"

#include <algorithm>
#include <iterator>
#include <string>

template <typename InputIterator1, typename InputIterator2>
bool range_equal(InputIterator1 first1, InputIterator1 last1,
                 InputIterator2 first2, InputIterator2 last2) {
  while (first1 != last1 && first2 != last2) {
    if (*first1 != *first2)
      return false;
    ++first1;
    ++first2;
  }
  return (first1 == last1) && (first2 == last2);
}

// Thanks: https://stackoverflow.com/a/15119347/1153203
bool compare_files(const std::string &filename1, const std::string &filename2) {
  // TODO: Fail comparison (or crash) if file paths are invalid (e.g. file
  // doesn't exist)
  std::ifstream file1(filename1);
  std::ifstream file2(filename2);

  std::istreambuf_iterator<char> begin1(file1);
  std::istreambuf_iterator<char> begin2(file2);

  std::istreambuf_iterator<char> end;

  auto x = range_equal(begin1, end, begin2, end);
  return x;
}