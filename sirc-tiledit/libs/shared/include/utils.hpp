
#ifndef UTILS_HPP
#define UTILS_HPP

#include <algorithm>
#include <map>
#include <ranges>
#include <type_traits>
#include <vector>

template <typename T>
std::vector<T> concatVecs(const std::vector<T> first,
                          const std::vector<T> second) {
  std::vector out(first.begin(), first.end());
  out.insert(out.end(), second.begin(), second.end());
  return out;
}

template <typename E> constexpr auto to_underlying(E e) noexcept {
  return static_cast<std::underlying_type_t<E>>(e);
}

template <typename T, typename U>
U findOrDefault(const std::map<const T, U> &map, const T key) {
  const auto result = map.find(key);
  if (result == map.end()) {
    return U();
  }
  return result->second;
}

template <typename U, typename T>
std::map<const T, U> spanToMapOfIndexes(const std::span<T> &items) {
  static_assert(std::is_integral_v<U>, "Indexes can only be numbers");
  std::map<T, U> out;
  std::ranges::transform(
      items, std::inserter(out, out.end()),
      [i = 0](T item) mutable { return std::pair(item, static_cast<U>(i++)); });
  return out;
}

#endif // UTILS_HPP
