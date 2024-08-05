
#ifndef UTILS_HPP
#define UTILS_HPP

#include <algorithm>
#include <unordered_map>
#include <vector>

template <typename T>
std::vector<T> concatVecs(const std::vector<T> first,
                          const std::vector<T> second) {
  std::vector out(first.begin(), first.end());
  out.insert(out.end(), second.begin(), second.end());
  return out;
}

template <typename T, typename U>
std::vector<std::pair<T, U>> pairWithValue(const std::vector<T> &originalVec,
                                           U valueToPairWith) {
  std::vector<std::pair<T, U>> paired;
  paired.reserve(originalVec.size());
  std::ranges::transform(originalVec, std::back_inserter(paired),
                         [valueToPairWith](T originalValue) {
                           return std::pair(originalValue, valueToPairWith);
                         });
  return paired;
}

#include <type_traits>

template <typename E> constexpr auto to_underlying(E e) noexcept {
  return static_cast<std::underlying_type_t<E>>(e);
}

template <typename T, typename U>
U findOrDefault(const std::unordered_map<T, U> &map, const T key) {
  const auto result = map.find(key);
  if (result == map.end()) {
    return U();
  }
  return result->second;
}

#endif // UTILS_HPP
