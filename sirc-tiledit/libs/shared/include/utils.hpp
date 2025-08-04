
#ifndef UTILS_HPP
#define UTILS_HPP

#include <algorithm>
#include <map>
#include <ranges>
#include <type_traits>
#include <utility>
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

// Polyfill for the c++23 `std::views::enumerate` function not available in
// c++20
template <std::ranges::input_range Range>
constexpr auto enumerate(Range &&range) {
  struct iterator {
    std::ranges::iterator_t<Range> it;
    std::size_t idx;
    bool operator!=(const iterator &other) const { return it != other.it; }
    void operator++() {
      ++it;
      ++idx;
    }
    auto operator*() const { return std::make_pair(idx, *it); }
  };

  struct view {
    Range range;
    auto begin() { return iterator{std::ranges::begin(range), 0}; }
    auto end() {
      return iterator{std::ranges::end(range),
                      static_cast<std::size_t>(std::ranges::distance(range))};
    }
  };

  return view{std::forward<Range>(range)};
}

#endif // UTILS_HPP
