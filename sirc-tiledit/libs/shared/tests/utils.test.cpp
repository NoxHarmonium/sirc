
#include <vector>

#include <utils.hpp>

#include "catch2/catch_amalgamated.hpp"

TEST_CASE("Converts vector correctly") {
  const std::vector<size_t> input = {1, 2, 3, 4, 5};

  const auto output = safeCastIntVector<uint16_t>(std::span(input));

  REQUIRE(output == std::vector<uint16_t>({1, 2, 3, 4, 5}));
}

TEST_CASE("Converts array correctly") {
  const std::array<size_t, 5> input = {1, 2, 3, 4, 5};

  const auto output =
      safeCastIntVector<uint16_t, const size_t>(std::span(input));

  REQUIRE(output == std::vector<uint16_t>({1, 2, 3, 4, 5}));
}

TEST_CASE("Throws when values are out of range") {
  const std::vector<size_t> input = {1, 2, 0xFFFFFFFF, 4, 5};

  const auto doTest = [&input]() {
    const auto output = safeCastIntVector<uint16_t>(std::span(input));
  };

  REQUIRE_THROWS_AS(doTest(), std::invalid_argument);
}
