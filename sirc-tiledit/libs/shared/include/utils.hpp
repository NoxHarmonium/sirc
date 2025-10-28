
#ifndef UTILS_HPP
#define UTILS_HPP

#include <algorithm>
#include <assert.h>
#include <concepts>
#include <format>
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

template <std::integral T, std::integral U>
std::vector<T> safeCastIntVector(const std::span<U> &in) {
  std::vector<T> out;
  out.reserve(in.size());
  std::ranges::transform(in, std::back_inserter(out), [](const auto &val) {
    if (val < static_cast<U>(std::numeric_limits<T>::min()) ||
        val > static_cast<U>(std::numeric_limits<T>::max())) {
      throw std::invalid_argument(
          "Integer value out of range when converting vector");
    }
    return static_cast<T>(val);
  });
  return out;
}

/**
 * Takes a span of integers and packs them into a smaller number of integers.
 *
 * @tparam T the output type
 * @tparam U the input type
 * @param in the span to pack into the output vector
 * @param bits the number of bits each value in the input span will be packed to
 * @return a vector of packed integers.
 */
template <std::integral T, std::integral U>
std::vector<T> packIntVector(const std::span<U> &in, const uint bits) {

  // Basic check that the number of bits for each value fits in the output type
  if (bits > std::numeric_limits<T>::digits) {
    throw std::invalid_argument(
        std::format("The number of specified bits ({}) does not fit inside the "
                    "output type ({})",
                    bits, typeid(T).name()));
  }
  // Check that the number of bits for each value divides evenly into the output
  if (std::numeric_limits<T>::digits % bits != 0) {
    throw std::invalid_argument(
        std::format("The number of specified bits ({}) does not fit "
                    "evenly into the number bits in the output type ({})",
                    bits, std::numeric_limits<T>::digits));
  }
  auto const valuesPerOutput = std::numeric_limits<T>::digits / bits;
  if (in.size() % valuesPerOutput != 0) {
    // We don't do any padding, each n input values must pack into a single
    // output value
    throw std::invalid_argument(std::format(
        "The number of values in the input ({}) does not divide evenly by the "
        "number of values that will be packed into the output ({})",
        in.size(), valuesPerOutput));
  }

  std::vector<T> out;
  out.reserve(in.size() / valuesPerOutput);

  for (uint index = 0; index < in.size(); index += valuesPerOutput) {
    T outValue = static_cast<T>(0);
    for (uint i = 0; i < valuesPerOutput; ++i) {
      const auto val = in[index + i];

      // Shift the most significant "chunk" first so that they don't interfere
      // with each other
      auto const shift = (valuesPerOutput - 1 - i) * bits;
      auto const shifted = val << shift;
      outValue |= shifted;
    }

    out.push_back(static_cast<T>(outValue));
  }

  return out;
}

#endif // UTILS_HPP
